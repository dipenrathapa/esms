#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]

mod api;
mod background;
mod business;
mod config;
mod error;
mod models;
mod retry;
mod sensor;
mod state;
mod storage;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use mysql_async::{Opts, Pool};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::{sync::Mutex, time::Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

use api::{get_fhir_observation, get_history, get_realtime, get_redis_history, health};
use background::sensor_task;
use config::AppConfig;
use retry::RetryConfig;
use state::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let config = AppConfig::from_env_validated().map_err(|e| {
        error!(error = %e, "Configuration validation failed");
        std::io::Error::other(e.to_string())
    })?;

    info!(
        operation = "application_startup",
        use_serial = %config.use_serial,
        bind_addr = %config.bind_addr,
        serial_tcp_host = %config.serial_tcp_host,
        serial_tcp_port = %config.serial_tcp_port,
        "Starting ESMS backend"
    );

    // Create cancellation token for graceful shutdown
    let shutdown_token = CancellationToken::new();

    // Initialize Redis
    let redis = redis::Client::open(config.redis_url.clone()).map_err(|e| {
        error!(
            error = %e,
            operation = "redis_init",
            "Failed to initialize Redis client"
        );
        std::io::Error::other(format!("Redis init failed: {e}"))
    })?;

    info!(
        operation = "redis_initialized",
        "Redis client initialized successfully"
    );

    // Initialize MySQL
    let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

    info!(
        operation = "mysql_initialized",
        "MySQL connection pool initialized successfully"
    );

    // Create app state
    let state = web::Data::new(AppState {
        redis: Arc::new(Mutex::new(redis)),
        mysql,
        memory: Arc::new(Mutex::new(VecDeque::new())),
        config: config.clone(),
        shutdown_token: shutdown_token.clone(),
        retry_config: RetryConfig::default(),
    });

    // Spawn background sensor task with shutdown token
    let sensor_task_handle = tokio::spawn(sensor_task(state.clone(), shutdown_token.child_token()));

    // Create HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .route("/health", web::get().to(health))
            .route("/api/realtime", web::get().to(get_realtime))
            .route("/api/redis", web::get().to(get_redis_history))
            .route("/api/history", web::get().to(get_history))
            .route("/api/fhir/observation", web::get().to(get_fhir_observation))
    })
    .bind(&config.bind_addr)?
    .run();

    info!(
        operation = "http_server_started",
        bind_addr = %config.bind_addr,
        "HTTP server is running"
    );

    let server_handle = server.handle();

    // Setup graceful shutdown signal handler
    let shutdown_signal = async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl-c");

        info!(
            operation = "shutdown_signal_received",
            "Shutdown signal received, initiating graceful shutdown..."
        );

        // Trigger cancellation token to stop background tasks
        shutdown_token.cancel();

        // Stop HTTP server gracefully
        server_handle.stop(true).await;

        info!(operation = "http_server_stopped", "HTTP server stopped");
    };

    // Run server and wait for shutdown signal
    tokio::select! {
        result = server => {
            result?;
        }
        () = shutdown_signal => {
            info!(
                operation = "shutdown_signal_handled",
                "Shutdown signal handled"
            );
        }
    }

    // Wait for background task to complete
    match tokio::time::timeout(Duration::from_secs(10), sensor_task_handle).await {
        Ok(Ok(())) => info!(
            operation = "background_task_stopped",
            "Background task stopped successfully"
        ),
        Ok(Err(e)) => error!(
            error = ?e,
            operation = "background_task_error",
            "Background task encountered an error during shutdown"
        ),
        Err(_) => error!(
            operation = "background_task_timeout",
            timeout_seconds = 10,
            "Background task did not stop within timeout"
        ),
    }

    info!(
        operation = "application_shutdown_complete",
        "Application shutdown complete"
    );
    Ok(())
}
