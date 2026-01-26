// hello
// use actix_cors::Cors;
// use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::{prelude::Queryable, Opts, Pool};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::{collections::VecDeque, env, sync::Arc};
// use tokio::{
//     io::{AsyncBufReadExt, BufReader},
//     net::TcpStream,
//     sync::Mutex,
//     time::{interval, Duration},
// };
// use tracing::{info, warn};
// use tracing_subscriber::{fmt, EnvFilter};
// use validator::Validate;

// // ======================================================
// // Configuration
// // ======================================================

// #[derive(Clone)]
// struct AppConfig {
//     redis_url: String,
//     mysql_url: String,
//     bind_addr: String,
//     use_serial: bool,
//     serial_tcp_host: String,
//     serial_tcp_port: u16,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         let use_serial = env::var("USE_SERIAL")
//             .unwrap_or_else(|_| "true".to_string())
//             .parse::<bool>()
//             .unwrap_or(true);

//         let serial_tcp_port = env::var("SERIAL_TCP_PORT")
//             .unwrap_or_else(|_| "5555".to_string())
//             .parse::<u16>()
//             .unwrap_or(5555);

//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//             use_serial,
//             serial_tcp_host: env::var("SERIAL_TCP_HOST")
//                 .unwrap_or_else(|_| "host.docker.internal".to_string()),
//             serial_tcp_port,
//         }
//     }
// }

// // ======================================================
// // Error Handling
// // ======================================================

// #[allow(dead_code)]
// #[derive(thiserror::Error, Debug)]
// enum ApiError {
//     #[error("Internal server error")]
//     Internal,
// }

// impl actix_web::ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         HttpResponse::InternalServerError().json(serde_json::json!({
//             "error": "internal_error"
//         }))
//     }
// }

// // ======================================================
// // Models
// // ======================================================

// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// struct SensorData {
//     #[validate(range(min = 0.0, max = 60.0))]
//     temperature: f64,

//     #[validate(range(min = 0.0, max = 100.0))]
//     humidity: f64,

//     #[validate(range(min = 0.0, max = 120.0))]
//     noise: f64,

//     #[validate(range(min = 30.0, max = 200.0))]
//     heart_rate: f64,

//     motion: bool,
//     timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// struct EnhancedSensorData {
//     #[serde(flatten)]
//     data: SensorData,
//     stress_index: f64,
//     stress_level: String,
// }

// // ======================================================
// // App State
// // ======================================================

// struct AppState {
//     redis: Arc<Mutex<redis::Client>>,
//     mysql: Pool,
//     memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
//     config: AppConfig,
// }

// // ======================================================
// // Business Logic
// // ======================================================

// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;

//     score.clamp(0.0, 1.0)
// }

// fn stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ======================================================
// // Sensor Simulation
// // ======================================================

// fn simulate_sensor_data() -> SensorData {
//     let mut rng = rand::thread_rng();
//     SensorData {
//         temperature: rng.gen_range(20.0..35.0),
//         humidity: rng.gen_range(40.0..80.0),
//         noise: rng.gen_range(50.0..90.0),
//         heart_rate: rng.gen_range(60.0..100.0),
//         motion: rng.gen_bool(0.3),
//         timestamp: Utc::now().to_rfc3339(),
//     }
// }

// // ======================================================
// // TCP Serial Reading (Mac/Docker compatible)
// // ======================================================

// async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
//     match TcpStream::connect((host, port)).await {
//         Ok(stream) => {
//             let mut reader = BufReader::new(stream);
//             let mut line = String::new();
//             match reader.read_line(&mut line).await {
//                 Ok(_) => {
//                     if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
//                         Some(sensor)
//                     } else {
//                         warn!("Failed to parse JSON from TCP: {}", line.trim());
//                         None
//                     }
//                 }
//                 Err(e) => {
//                     warn!("TCP read error: {:?}", e);
//                     None
//                 }
//             }
//         }
//         Err(e) => {
//             warn!("TCP connect failed: {:?}", e);
//             None
//         }
//     }
// }

// // ======================================================
// // Background Task
// // ======================================================

// async fn sensor_task(state: web::Data<AppState>) {
//     let mut ticker = interval(Duration::from_secs(1));

//     loop {
//         ticker.tick().await;

//         let data = if state.config.use_serial {
//             read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port)
//                 .await
//                 .unwrap_or_else(simulate_sensor_data)
//         } else {
//             simulate_sensor_data()
//         };

//         if let Err(e) = data.validate() {
//             warn!("Validation failed: {:?}", e);
//             continue;
//         }

//         let index = calculate_stress_index(&data);
//         let enhanced = EnhancedSensorData {
//             stress_index: index,
//             stress_level: stress_level(index),
//             data,
//         };

//         // In-memory fallback
//         {
//             let mut mem = state.memory.lock().await;
//             mem.push_back(enhanced.clone());
//             if mem.len() > 600 {
//                 mem.pop_front();
//             }
//         }

//         // Redis
//         let redis = state.redis.clone();
//         let redis_payload = enhanced.clone();
//         tokio::spawn(async move {
//             if let Ok(mut conn) = redis.lock().await.get_multiplexed_async_connection().await {
//                 if let Err(e) = conn
//                     .set_ex::<_, _, ()>(
//                         format!("sensor:{}", redis_payload.data.timestamp),
//                         serde_json::to_string(&redis_payload).unwrap(),
//                         600,
//                     )
//                     .await
//                 {
//                     warn!("Redis write failed: {:?}", e);
//                 }
//             }
//         });

//         // MySQL
//         let pool = state.mysql.clone();
//         let db_payload = enhanced.clone();
//         tokio::spawn(async move {
//             if let Ok(mut conn) = pool.get_conn().await {
//                 let res = conn
//                     .exec_drop(
//                         r#"INSERT INTO sensor_data
//                         (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//                         (
//                             db_payload.data.temperature,
//                             db_payload.data.humidity,
//                             db_payload.data.noise,
//                             db_payload.data.heart_rate,
//                             db_payload.data.motion,
//                             db_payload.stress_index,
//                             db_payload.stress_level,
//                             db_payload.data.timestamp,
//                         ),
//                     )
//                     .await;
//                 if let Err(e) = res {
//                     warn!("MySQL insert failed: {:?}", e);
//                 }
//             }
//         });
//     }
// }

// // ======================================================
// // API
// // ======================================================

// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// // ======================================================
// // Main
// // ======================================================

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .json()
//         .init();

//     let config = AppConfig::from_env();
//     info!("Starting ESMS backend (use_serial={})", config.use_serial);

//     let redis = redis::Client::open(config.redis_url.clone()).expect("Redis init failed");
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
//         config: config.clone(),
//     });

//     tokio::spawn(sensor_task(state.clone()));

//     HttpServer::new(move || {
//         App::new()
//             .wrap(Logger::default())
//             .wrap(Cors::permissive())
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//     })
//     .bind(&config.bind_addr)?
//     .run()
//     .await
// }

// use actix_cors::Cors;
// use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::{prelude::Queryable, Opts, Pool};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::{collections::VecDeque, env, sync::Arc};
// use tokio::{
//     io::{AsyncBufReadExt, BufReader},
//     net::TcpStream,
//     sync::Mutex,
//     time::{interval, Duration},
// };
// use tokio_util::sync::CancellationToken;
// use tracing::{error, info, warn};
// use tracing_subscriber::{fmt, EnvFilter};
// use validator::Validate;

// // ======================================================
// // Configuration
// // ======================================================
// #[derive(Clone)]
// struct AppConfig {
//     redis_url: String,
//     mysql_url: String,
//     bind_addr: String,
//     use_serial: bool,
//     serial_tcp_host: String,
//     serial_tcp_port: u16,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         let use_serial = env::var("USE_SERIAL")
//             .unwrap_or_else(|_| "true".to_string())
//             .parse::<bool>()
//             .unwrap_or(true);

//         let serial_tcp_port = env::var("SERIAL_TCP_PORT")
//             .unwrap_or_else(|_| "5555".to_string())
//             .parse::<u16>()
//             .unwrap_or(5555);

//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//             use_serial,
//             serial_tcp_host: env::var("SERIAL_TCP_HOST")
//                 .unwrap_or_else(|_| "host.docker.internal".to_string()),
//             serial_tcp_port,
//         }
//     }
// }

// // ======================================================
// // Error Handling (Centralized)
// // ======================================================
// #[derive(thiserror::Error, Debug)]
// enum ApiError {
//     #[error("Internal server error")]
//     Internal,

//     #[error("Database error: {0}")]
//     Database(String),

//     #[error("Redis error: {0}")]
//     Redis(String),

//     #[error("Validation error: {0}")]
//     Validation(String),

//     #[error("TCP connection error: {0}")]
//     TcpConnection(String),
// }

// impl actix_web::ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             ApiError::Internal => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "internal_error",
//                 "message": "An internal error occurred"
//             })),
//             ApiError::Database(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "database_error",
//                 "message": msg
//             })),
//             ApiError::Redis(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "redis_error",
//                 "message": msg
//             })),
//             ApiError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
//                 "error": "validation_error",
//                 "message": msg
//             })),
//             ApiError::TcpConnection(msg) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
//                 "error": "tcp_connection_error",
//                 "message": msg
//             })),
//         }
//     }
// }

// // Helper function for logging and converting errors
// fn log_and_convert_error<E: std::fmt::Display>(context: &str, error: E) -> ApiError {
//     error!("{}: {}", context, error);
//     ApiError::Internal
// }

// // ======================================================
// // Models
// // ======================================================
// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// struct SensorData {
//     #[validate(range(min = 0.0, max = 60.0))]
//     temperature: f64,
//     #[validate(range(min = 0.0, max = 100.0))]
//     humidity: f64,
//     #[validate(range(min = 0.0, max = 120.0))]
//     noise: f64,
//     #[validate(range(min = 30.0, max = 200.0))]
//     heart_rate: f64,
//     motion: bool,
//     timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// struct EnhancedSensorData {
//     #[serde(flatten)]
//     data: SensorData,
//     stress_index: f64,
//     stress_level: String,
// }

// // ======================================================
// // App State
// // ======================================================
// struct AppState {
//     redis: Arc<Mutex<redis::Client>>,
//     mysql: Pool,
//     memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
//     config: AppConfig,
//     shutdown_token: CancellationToken,
// }

// // ======================================================
// // Business Logic
// // ======================================================
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;
//     score.clamp(0.0, 1.0)
// }

// fn stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ======================================================
// // Sensor Simulation
// // ======================================================
// fn simulate_sensor_data() -> SensorData {
//     let mut rng = rand::thread_rng();
//     SensorData {
//         temperature: rng.gen_range(20.0..35.0),
//         humidity: rng.gen_range(40.0..80.0),
//         noise: rng.gen_range(50.0..90.0),
//         heart_rate: rng.gen_range(60.0..100.0),
//         motion: rng.gen_bool(0.3),
//         timestamp: Utc::now().to_rfc3339(),
//     }
// }

// // ======================================================
// // TCP Serial Reading (Mac/Docker compatible)
// // ======================================================
// async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
//     match TcpStream::connect((host, port)).await {
//         Ok(stream) => {
//             let mut reader = BufReader::new(stream);
//             let mut line = String::new();
//             match reader.read_line(&mut line).await {
//                 Ok(_) => {
//                     if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
//                         Some(sensor)
//                     } else {
//                         warn!("Failed to parse JSON from TCP: {}", line.trim());
//                         None
//                     }
//                 }
//                 Err(e) => {
//                     warn!("TCP read error: {:?}", e);
//                     None
//                 }
//             }
//         }
//         Err(e) => {
//             warn!("TCP connect failed: {:?}", e);
//             None
//         }
//     }
// }

// // ======================================================
// // Background Task with Graceful Shutdown
// // ======================================================
// async fn sensor_task(state: web::Data<AppState>, shutdown_token: CancellationToken) {
//     let mut ticker = interval(Duration::from_secs(1));

//     info!("Sensor task started");

//     loop {
//         tokio::select! {
//             _ = shutdown_token.cancelled() => {
//                 info!("Sensor task received shutdown signal, cleaning up...");
//                 break;
//             }
//             _ = ticker.tick() => {
//                 if let Err(e) = process_sensor_data(&state).await {
//                     error!("Error processing sensor data: {:?}", e);
//                     // Continue running even on errors
//                 }
//             }
//         }
//     }

//     info!("Sensor task stopped gracefully");
// }

// async fn process_sensor_data(state: &web::Data<AppState>) -> Result<(), ApiError> {
//     let data = if state.config.use_serial {
//         read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port)
//             .await
//             .unwrap_or_else(simulate_sensor_data)
//     } else {
//         simulate_sensor_data()
//     };

//     // Validate sensor data
//     if let Err(e) = data.validate() {
//         warn!("Validation failed: {:?}", e);
//         return Err(ApiError::Validation(format!("{:?}", e)));
//     }

//     let index = calculate_stress_index(&data);
//     let enhanced = EnhancedSensorData {
//         stress_index: index,
//         stress_level: stress_level(index),
//         data,
//     };

//     // In-memory fallback (always succeeds)
//     {
//         let mut mem = state.memory.lock().await;
//         mem.push_back(enhanced.clone());
//         if mem.len() > 600 {
//             mem.pop_front();
//         }
//     }

//     // Redis (non-blocking, fire and forget with error logging)
//     let redis = state.redis.clone();
//     let redis_payload = enhanced.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_redis(redis, redis_payload).await {
//             warn!("Redis storage failed: {:?}", e);
//         }
//     });

//     // MySQL (non-blocking, fire and forget with error logging)
//     let pool = state.mysql.clone();
//     let db_payload = enhanced.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_mysql(pool, db_payload).await {
//             warn!("MySQL storage failed: {:?}", e);
//         }
//     });

//     Ok(())
// }

// async fn store_to_redis(
//     redis: Arc<Mutex<redis::Client>>,
//     payload: EnhancedSensorData,
// ) -> Result<(), ApiError> {
//     let mut conn = redis
//         .lock()
//         .await
//         .get_multiplexed_async_connection()
//         .await
//         .map_err(|e| ApiError::Redis(format!("Connection failed: {}", e)))?;

//     let key = format!("sensor:{}", payload.data.timestamp);
//     let value = serde_json::to_string(&payload)
//         .map_err(|e| ApiError::Redis(format!("Serialization failed: {}", e)))?;

//     conn.set_ex::<_, _, ()>(key, value, 600)
//         .await
//         .map_err(|e| ApiError::Redis(format!("SET failed: {}", e)))?;

//     Ok(())
// }

// async fn store_to_mysql(pool: Pool, payload: EnhancedSensorData) -> Result<(), ApiError> {
//     let mut conn = pool
//         .get_conn()
//         .await
//         .map_err(|e| ApiError::Database(format!("Connection failed: {}", e)))?;

//     conn.exec_drop(
//         r#"INSERT INTO sensor_data
//            (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//         (
//             payload.data.temperature,
//             payload.data.humidity,
//             payload.data.noise,
//             payload.data.heart_rate,
//             payload.data.motion,
//             payload.stress_index,
//             payload.stress_level,
//             payload.data.timestamp,
//         ),
//     )
//     .await
//     .map_err(|e| ApiError::Database(format!("Insert failed: {}", e)))?;

//     Ok(())
// }

// // ======================================================
// // API
// // ======================================================
// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// // ======================================================
// // Main
// // ======================================================
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .json()
//         .init();

//     let config = AppConfig::from_env();
//     info!("Starting ESMS backend (use_serial={})", config.use_serial);

//     // Create cancellation token for graceful shutdown
//     let shutdown_token = CancellationToken::new();

//     // Initialize Redis
//     let redis = redis::Client::open(config.redis_url.clone())
//         .expect("Redis init failed");

//     // Initialize MySQL
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     // Create app state
//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
//         config: config.clone(),
//         shutdown_token: shutdown_token.clone(),
//     });

//     // Spawn background sensor task with shutdown token
//     let sensor_task_handle = tokio::spawn(sensor_task(state.clone(), shutdown_token.child_token()));

//     // Create HTTP server
//     let server = HttpServer::new(move || {
//         App::new()
//             .wrap(Logger::default())
//             .wrap(Cors::permissive())
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//     })
//     .bind(&config.bind_addr)?
//     .run();

//     info!("Server running on {}", config.bind_addr);

//     let server_handle = server.handle();

//     // Setup graceful shutdown signal handler
//     let shutdown_signal = async move {
//         tokio::signal::ctrl_c()
//             .await
//             .expect("Failed to listen for ctrl-c");

//         info!("Shutdown signal received, initiating graceful shutdown...");

//         // Trigger cancellation token to stop background tasks
//         shutdown_token.cancel();

//         // Stop HTTP server gracefully
//         server_handle.stop(true).await;

//         info!("HTTP server stopped");
//     };

//     // Run server and wait for shutdown signal
//     tokio::select! {
//         result = server => {
//             result?;
//         }
//         _ = shutdown_signal => {
//             info!("Shutdown signal handled");
//         }
//     }

//     // Wait for background task to complete
//     match tokio::time::timeout(Duration::from_secs(10), sensor_task_handle).await {
//         Ok(Ok(())) => info!("Background task stopped successfully"),
//         Ok(Err(e)) => error!("Background task error: {:?}", e),
//         Err(_) => error!("Background task did not stop within timeout"),
//     }

//     info!("Application shutdown complete");
//     Ok(())
// }

// use actix_cors::Cors;
// use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::{prelude::Queryable, Opts, Pool};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::{collections::VecDeque, env, sync::Arc};
// use tokio::{
//     io::{AsyncBufReadExt, BufReader},
//     net::TcpStream,
//     sync::Mutex,
//     time::{interval, Duration},
// };
// use tokio_util::sync::CancellationToken;
// use tracing::{error, info, warn};
// use tracing_subscriber::{fmt, EnvFilter};
// use validator::Validate;

// // ======================================================
// // Configuration
// // ======================================================
// #[derive(Clone)]
// struct AppConfig {
//     redis_url: String,
//     mysql_url: String,
//     bind_addr: String,
//     use_serial: bool,
//     serial_tcp_host: String,
//     serial_tcp_port: u16,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         let use_serial = env::var("USE_SERIAL")
//             .unwrap_or_else(|_| "true".to_string())
//             .parse::<bool>()
//             .unwrap_or(true);

//         let serial_tcp_port = env::var("SERIAL_TCP_PORT")
//             .unwrap_or_else(|_| "5555".to_string())
//             .parse::<u16>()
//             .unwrap_or(5555);

//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//             use_serial,
//             serial_tcp_host: env::var("SERIAL_TCP_HOST")
//                 .unwrap_or_else(|_| "host.docker.internal".to_string()),
//             serial_tcp_port,
//         }
//     }
// }

// // ======================================================
// // Error Handling (Centralized)
// // ======================================================
// #[derive(thiserror::Error, Debug)]
// enum ApiError {
//     #[error("Internal server error")]
//     Internal,

//     #[error("Database error: {0}")]
//     Database(String),

//     #[error("Redis error: {0}")]
//     Redis(String),

//     #[error("Validation error: {0}")]
//     Validation(String),

//     #[error("TCP connection error: {0}")]
//     TcpConnection(String),
// }

// impl actix_web::ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             ApiError::Internal => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "internal_error",
//                 "message": "An internal error occurred"
//             })),
//             ApiError::Database(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "database_error",
//                 "message": msg
//             })),
//             ApiError::Redis(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "redis_error",
//                 "message": msg
//             })),
//             ApiError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
//                 "error": "validation_error",
//                 "message": msg
//             })),
//             ApiError::TcpConnection(msg) => HttpResponse::ServiceUnavailable().json(serde_json::json!({
//                 "error": "tcp_connection_error",
//                 "message": msg
//             })),
//         }
//     }
// }

// // Helper function for logging and converting errors
// fn log_and_convert_error<E: std::fmt::Display>(context: &str, error: E) -> ApiError {
//     error!("{}: {}", context, error);
//     ApiError::Internal
// }

// // ======================================================
// // Models
// // ======================================================
// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// struct SensorData {
//     #[validate(range(min = 0.0, max = 60.0))]
//     temperature: f64,
//     #[validate(range(min = 0.0, max = 100.0))]
//     humidity: f64,
//     #[validate(range(min = 0.0, max = 120.0))]
//     noise: f64,
//     #[validate(range(min = 30.0, max = 200.0))]
//     heart_rate: f64,
//     motion: bool,
//     timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// struct EnhancedSensorData {
//     #[serde(flatten)]
//     data: SensorData,
//     stress_index: f64,
//     stress_level: String,
// }

// // ======================================================
// // App State
// // ======================================================
// struct AppState {
//     redis: Arc<Mutex<redis::Client>>,
//     mysql: Pool,
//     memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
//     config: AppConfig,
//     shutdown_token: CancellationToken,
// }

// // ======================================================
// // Business Logic
// // ======================================================
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;
//     score.clamp(0.0, 1.0)
// }

// fn stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ======================================================
// // Sensor Simulation
// // ======================================================
// fn simulate_sensor_data() -> SensorData {
//     let mut rng = rand::thread_rng();
//     SensorData {
//         temperature: rng.gen_range(20.0..35.0),
//         humidity: rng.gen_range(40.0..80.0),
//         noise: rng.gen_range(50.0..90.0),
//         heart_rate: rng.gen_range(60.0..100.0),
//         motion: rng.gen_bool(0.3),
//         timestamp: Utc::now().to_rfc3339(),
//     }
// }

// // ======================================================
// // TCP Serial Reading (Mac/Docker compatible)
// // ======================================================
// async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
//     match TcpStream::connect((host, port)).await {
//         Ok(stream) => {
//             info!(
//                 operation = "tcp_connect",
//                 host = %host,
//                 port = %port,
//                 "Successfully connected to TCP sensor stream"
//             );

//             let mut reader = BufReader::new(stream);
//             let mut line = String::new();
//             match reader.read_line(&mut line).await {
//                 Ok(bytes_read) => {
//                     if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
//                         info!(
//                             operation = "tcp_read",
//                             host = %host,
//                             port = %port,
//                             bytes_read = %bytes_read,
//                             temperature = %sensor.temperature,
//                             heart_rate = %sensor.heart_rate,
//                             "Successfully parsed sensor data from TCP"
//                         );
//                         Some(sensor)
//                     } else {
//                         warn!(
//                             operation = "tcp_parse",
//                             host = %host,
//                             port = %port,
//                             raw_data = %line.trim(),
//                             "Failed to parse JSON from TCP stream"
//                         );
//                         None
//                     }
//                 }
//                 Err(e) => {
//                     error!(
//                         error = %e,
//                         operation = "tcp_read",
//                         host = %host,
//                         port = %port,
//                         "Failed to read data from TCP stream"
//                     );
//                     None
//                 }
//             }
//         }
//         Err(e) => {
//             error!(
//                 error = %e,
//                 operation = "tcp_connect",
//                 host = %host,
//                 port = %port,
//                 "Failed to connect to TCP sensor stream"
//             );
//             None
//         }
//     }
// }

// // ======================================================
// // Background Task with Graceful Shutdown
// // ======================================================
// async fn sensor_task(state: web::Data<AppState>, shutdown_token: CancellationToken) {
//     let mut ticker = interval(Duration::from_secs(1));

//     info!(
//         operation = "sensor_task_start",
//         use_serial = %state.config.use_serial,
//         serial_host = %state.config.serial_tcp_host,
//         serial_port = %state.config.serial_tcp_port,
//         "Sensor background task started"
//     );

//     loop {
//         tokio::select! {
//             _ = shutdown_token.cancelled() => {
//                 info!(
//                     operation = "sensor_task_shutdown",
//                     "Sensor task received shutdown signal, cleaning up..."
//                 );
//                 break;
//             }
//             _ = ticker.tick() => {
//                 if let Err(e) = process_sensor_data(&state).await {
//                     error!(
//                         error = ?e,
//                         operation = "sensor_task_process",
//                         "Error processing sensor data in background task"
//                     );
//                     // Continue running even on errors
//                 }
//             }
//         }
//     }

//     info!(
//         operation = "sensor_task_stopped",
//         "Sensor task stopped gracefully"
//     );
// }

// async fn process_sensor_data(state: &web::Data<AppState>) -> Result<(), ApiError> {
//     let data = if state.config.use_serial {
//         match read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port).await {
//             Some(sensor_data) => {
//                 info!(
//                     operation = "sensor_data_source",
//                     source = "tcp",
//                     "Using real sensor data from TCP stream"
//                 );
//                 sensor_data
//             }
//             None => {
//                 warn!(
//                     operation = "sensor_data_source",
//                     source = "simulation_fallback",
//                     "TCP read failed, falling back to simulated data"
//                 );
//                 simulate_sensor_data()
//             }
//         }
//     } else {
//         info!(
//             operation = "sensor_data_source",
//             source = "simulation",
//             "Using simulated sensor data"
//         );
//         simulate_sensor_data()
//     };

//     // Validate sensor data
//     if let Err(e) = data.validate() {
//         warn!(
//             operation = "sensor_validation",
//             error = ?e,
//             temperature = %data.temperature,
//             humidity = %data.humidity,
//             heart_rate = %data.heart_rate,
//             "Sensor data validation failed"
//         );
//         return Err(ApiError::Validation(format!("{:?}", e)));
//     }

//     let index = calculate_stress_index(&data);
//     let enhanced = EnhancedSensorData {
//         stress_index: index,
//         stress_level: stress_level(index),
//         data,
//     };

//     // In-memory fallback (always succeeds)
//     {
//         let mut mem = state.memory.lock().await;
//         mem.push_back(enhanced.clone());
//         if mem.len() > 600 {
//             mem.pop_front();
//         }
//         info!(
//             operation = "memory_store",
//             buffer_size = %mem.len(),
//             timestamp = %enhanced.data.timestamp,
//             stress_level = %enhanced.stress_level,
//             "Stored sensor data in memory buffer"
//         );
//     }

//     // Redis (non-blocking, fire and forget with error logging)
//     let redis = state.redis.clone();
//     let redis_payload = enhanced.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_redis(redis, redis_payload).await {
//             // Error already logged in store_to_redis
//             warn!(
//                 error = ?e,
//                 operation = "background_redis_store",
//                 "Redis background task failed"
//             );
//         }
//     });

//     // MySQL (non-blocking, fire and forget with error logging)
//     let pool = state.mysql.clone();
//     let db_payload = enhanced.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_mysql(pool, db_payload).await {
//             // Error already logged in store_to_mysql
//             warn!(
//                 error = ?e,
//                 operation = "background_mysql_store",
//                 "MySQL background task failed"
//             );
//         }
//     });

//     Ok(())
// }

// async fn store_to_redis(
//     redis: Arc<Mutex<redis::Client>>,
//     payload: EnhancedSensorData,
// ) -> Result<(), ApiError> {
//     let timestamp = payload.data.timestamp.clone();

//     let mut conn = redis
//         .lock()
//         .await
//         .get_multiplexed_async_connection()
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_connection",
//                 timestamp = %timestamp,
//                 "Failed to establish Redis connection"
//             );
//             ApiError::Redis(format!("Connection failed: {}", e))
//         })?;

//     let key = format!("sensor:{}", timestamp);
//     let value = serde_json::to_string(&payload)
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_serialization",
//                 timestamp = %timestamp,
//                 key = %key,
//                 "Failed to serialize sensor data for Redis"
//             );
//             ApiError::Redis(format!("Serialization failed: {}", e))
//         })?;

//     conn.set_ex::<_, _, ()>(key.clone(), value, 600)
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_set",
//                 timestamp = %timestamp,
//                 key = %key,
//                 ttl = 600,
//                 "Failed to set value in Redis"
//             );
//             ApiError::Redis(format!("SET failed: {}", e))
//         })?;

//     info!(
//         operation = "redis_set",
//         timestamp = %timestamp,
//         key = %key,
//         stress_level = %payload.stress_level,
//         "Successfully stored sensor data in Redis"
//     );

//     Ok(())
// }

// async fn store_to_mysql(pool: Pool, payload: EnhancedSensorData) -> Result<(), ApiError> {
//     let timestamp = payload.data.timestamp.clone();

//     let mut conn = pool
//         .get_conn()
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "mysql_connection",
//                 timestamp = %timestamp,
//                 "Failed to get MySQL connection from pool"
//             );
//             ApiError::Database(format!("Connection failed: {}", e))
//         })?;

//     conn.exec_drop(
//         r#"INSERT INTO sensor_data
//            (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//         (
//             payload.data.temperature,
//             payload.data.humidity,
//             payload.data.noise,
//             payload.data.heart_rate,
//             payload.data.motion,
//             payload.stress_index,
//             payload.stress_level.clone(),
//             timestamp.clone(),
//         ),
//     )
//     .await
//     .map_err(|e| {
//         error!(
//             error = %e,
//             operation = "mysql_insert",
//             timestamp = %timestamp,
//             temperature = %payload.data.temperature,
//             humidity = %payload.data.humidity,
//             heart_rate = %payload.data.heart_rate,
//             stress_level = %payload.stress_level,
//             "Failed to insert sensor data into MySQL"
//         );
//         ApiError::Database(format!("Insert failed: {}", e))
//     })?;

//     info!(
//         operation = "mysql_insert",
//         timestamp = %timestamp,
//         stress_level = %payload.stress_level,
//         stress_index = %payload.stress_index,
//         "Successfully stored sensor data in MySQL"
//     );

//     Ok(())
// }

// // ======================================================
// // API
// // ======================================================
// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// // ======================================================
// // Main
// // ======================================================
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .json()
//         .init();

//     let config = AppConfig::from_env();

//     info!(
//         operation = "application_startup",
//         use_serial = %config.use_serial,
//         bind_addr = %config.bind_addr,
//         serial_tcp_host = %config.serial_tcp_host,
//         serial_tcp_port = %config.serial_tcp_port,
//         "Starting ESMS backend"
//     );

//     // Create cancellation token for graceful shutdown
//     let shutdown_token = CancellationToken::new();

//     // Initialize Redis
//     let redis = redis::Client::open(config.redis_url.clone())
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_init",
//                 "Failed to initialize Redis client"
//             );
//             std::io::Error::new(std::io::ErrorKind::Other, format!("Redis init failed: {}", e))
//         })?;

//     info!(
//         operation = "redis_initialized",
//         "Redis client initialized successfully"
//     );

//     // Initialize MySQL
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     info!(
//         operation = "mysql_initialized",
//         "MySQL connection pool initialized successfully"
//     );

//     // Create app state
//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
//         config: config.clone(),
//         shutdown_token: shutdown_token.clone(),
//     });

//     // Spawn background sensor task with shutdown token
//     let sensor_task_handle = tokio::spawn(sensor_task(state.clone(), shutdown_token.child_token()));

//     // Create HTTP server
//     let server = HttpServer::new(move || {
//         App::new()
//             .wrap(Logger::default())
//             .wrap(Cors::permissive())
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//     })
//     .bind(&config.bind_addr)?
//     .run();

//     info!(
//         operation = "http_server_started",
//         bind_addr = %config.bind_addr,
//         "HTTP server is running"
//     );

//     let server_handle = server.handle();

//     // Setup graceful shutdown signal handler
//     let shutdown_signal = async move {
//         tokio::signal::ctrl_c()
//             .await
//             .expect("Failed to listen for ctrl-c");

//         info!(
//             operation = "shutdown_signal_received",
//             "Shutdown signal received, initiating graceful shutdown..."
//         );

//         // Trigger cancellation token to stop background tasks
//         shutdown_token.cancel();

//         // Stop HTTP server gracefully
//         server_handle.stop(true).await;

//         info!(
//             operation = "http_server_stopped",
//             "HTTP server stopped"
//         );
//     };

//     // Run server and wait for shutdown signal
//     tokio::select! {
//         result = server => {
//             result?;
//         }
//         _ = shutdown_signal => {
//             info!(
//                 operation = "shutdown_signal_handled",
//                 "Shutdown signal handled"
//             );
//         }
//     }

//     // Wait for background task to complete
//     match tokio::time::timeout(Duration::from_secs(10), sensor_task_handle).await {
//         Ok(Ok(())) => info!(
//             operation = "background_task_stopped",
//             "Background task stopped successfully"
//         ),
//         Ok(Err(e)) => error!(
//             error = ?e,
//             operation = "background_task_error",
//             "Background task encountered an error during shutdown"
//         ),
//         Err(_) => error!(
//             operation = "background_task_timeout",
//             timeout_seconds = 10,
//             "Background task did not stop within timeout"
//         ),
//     }

//     info!(
//         operation = "application_shutdown_complete",
//         "Application shutdown complete"
//     );
//     Ok(())
// }

// use actix_cors::Cors;
// use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::{prelude::Queryable, Opts, Pool};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::{collections::VecDeque, env, net::SocketAddr, sync::Arc};
// use tokio::{
//     io::{AsyncBufReadExt, BufReader},
//     net::TcpStream,
//     sync::Mutex,
//     time::{interval, sleep, Duration},
// };
// use tokio_util::sync::CancellationToken;
// use tracing::{error, info, warn};
// use tracing_subscriber::{fmt, EnvFilter};
// use url::Url;
// use validator::Validate;

// // ======================================================
// // Configuration
// // ======================================================
// #[derive(Clone)]
// struct AppConfig {
//     redis_url: String,
//     mysql_url: String,
//     bind_addr: String,
//     use_serial: bool,
//     serial_tcp_host: String,
//     serial_tcp_port: u16,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         let use_serial = env::var("USE_SERIAL")
//             .unwrap_or_else(|_| "true".to_string())
//             .parse::<bool>()
//             .unwrap_or(true);

//         let serial_tcp_port = env::var("SERIAL_TCP_PORT")
//             .unwrap_or_else(|_| "5555".to_string())
//             .parse::<u16>()
//             .unwrap_or(5555);

//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//             use_serial,
//             serial_tcp_host: env::var("SERIAL_TCP_HOST")
//                 .unwrap_or_else(|_| "host.docker.internal".to_string()),
//             serial_tcp_port,
//         }
//     }

//     pub fn from_env_validated() -> Result<Self, ConfigError> {
//         let config = Self::from_env();
//         validate_config(&config)?;
//         Ok(config)
//     }
// }

// // ======================================================
// // Error Handling (Centralized)
// // ======================================================
// #[derive(thiserror::Error, Debug)]
// enum ApiError {
//     #[error("Internal server error")]
//     Internal,

//     #[error("Database error: {0}")]
//     Database(String),

//     #[error("Redis error: {0}")]
//     Redis(String),

//     #[error("Validation error: {0}")]
//     Validation(String),

//     #[error("TCP connection error: {0}")]
//     TcpConnection(String),
// }

// impl actix_web::ResponseError for ApiError {
//     fn error_response(&self) -> HttpResponse {
//         match self {
//             ApiError::Internal => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "internal_error",
//                 "message": "An internal error occurred"
//             })),
//             ApiError::Database(msg) => {
//                 HttpResponse::InternalServerError().json(serde_json::json!({
//                     "error": "database_error",
//                     "message": msg
//                 }))
//             }
//             ApiError::Redis(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
//                 "error": "redis_error",
//                 "message": msg
//             })),
//             ApiError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
//                 "error": "validation_error",
//                 "message": msg
//             })),
//             ApiError::TcpConnection(msg) => {
//                 HttpResponse::ServiceUnavailable().json(serde_json::json!({
//                     "error": "tcp_connection_error",
//                     "message": msg
//                 }))
//             }
//         }
//     }
// }

// // ======================================================
// // Configuration Error
// // ======================================================
// #[derive(thiserror::Error, Debug)]
// pub enum ConfigError {
//     #[error("Invalid Redis URL: {0}")]
//     InvalidRedisUrl(String),

//     #[error("Invalid MySQL URL: {0}")]
//     InvalidMysqlUrl(String),

//     #[error("Invalid bind address: {0}")]
//     InvalidBindAddr(String),

//     #[error("Invalid serial TCP configuration: {0}")]
//     InvalidSerialConfig(String),
// }

// // ======================================================
// // Configuration Validation
// // ======================================================
// fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
//     info!(
//         operation = "config_validation_start",
//         "Starting configuration validation"
//     );

//     validate_redis_url(&config.redis_url)?;
//     validate_mysql_url(&config.mysql_url)?;
//     validate_bind_addr(&config.bind_addr)?;

//     if config.use_serial {
//         validate_serial_config(&config.serial_tcp_host, config.serial_tcp_port)?;
//     }

//     info!(
//         operation = "config_validation_complete",
//         "Configuration validation successful"
//     );

//     Ok(())
// }

// fn validate_redis_url(url: &str) -> Result<(), ConfigError> {
//     if url.is_empty() {
//         return Err(ConfigError::InvalidRedisUrl(
//             "Redis URL is empty".to_string(),
//         ));
//     }

//     if !url.starts_with("redis://") && !url.starts_with("rediss://") {
//         return Err(ConfigError::InvalidRedisUrl(format!(
//             "Redis URL must start with redis:// or rediss://, got: {}",
//             url
//         )));
//     }

//     info!(
//         operation = "config_validation",
//         component = "redis_url",
//         "Redis URL validated successfully"
//     );

//     Ok(())
// }

// fn validate_mysql_url(url: &str) -> Result<(), ConfigError> {
//     if url.is_empty() {
//         return Err(ConfigError::InvalidMysqlUrl(
//             "MySQL URL is empty".to_string(),
//         ));
//     }

//     if !url.starts_with("mysql://") {
//         return Err(ConfigError::InvalidMysqlUrl(format!(
//             "MySQL URL must start with mysql://, got: {}",
//             url
//         )));
//     }

//     Url::parse(url)
//         .map_err(|e| ConfigError::InvalidMysqlUrl(format!("Invalid URL format: {}", e)))?;

//     info!(
//         operation = "config_validation",
//         component = "mysql_url",
//         "MySQL URL validated successfully"
//     );

//     Ok(())
// }

// fn validate_bind_addr(addr: &str) -> Result<(), ConfigError> {
//     if addr.is_empty() {
//         return Err(ConfigError::InvalidBindAddr(
//             "Bind address is empty".to_string(),
//         ));
//     }

//     addr.parse::<SocketAddr>().map_err(|e| {
//         ConfigError::InvalidBindAddr(format!("Invalid socket address format: {}", e))
//     })?;

//     info!(
//         operation = "config_validation",
//         component = "bind_addr",
//         bind_addr = %addr,
//         "Bind address validated successfully"
//     );

//     Ok(())
// }

// fn validate_serial_config(host: &str, port: u16) -> Result<(), ConfigError> {
//     if host.is_empty() {
//         return Err(ConfigError::InvalidSerialConfig(
//             "Serial TCP host is empty".to_string(),
//         ));
//     }

//     if port == 0 || port > 65535 {
//         return Err(ConfigError::InvalidSerialConfig(format!(
//             "Serial TCP port {} is out of valid range (1-65535)",
//             port
//         )));
//     }

//     info!(
//         operation = "config_validation",
//         component = "serial_tcp",
//         host = %host,
//         port = %port,
//         "Serial TCP configuration validated successfully"
//     );

//     Ok(())
// }

// // ======================================================
// // Models
// // ======================================================
// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// struct SensorData {
//     #[validate(range(min = 0.0, max = 60.0))]
//     temperature: f64,
//     #[validate(range(min = 0.0, max = 100.0))]
//     humidity: f64,
//     #[validate(range(min = 0.0, max = 120.0))]
//     noise: f64,
//     #[validate(range(min = 30.0, max = 200.0))]
//     heart_rate: f64,
//     motion: bool,
//     timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// struct EnhancedSensorData {
//     #[serde(flatten)]
//     data: SensorData,
//     stress_index: f64,
//     stress_level: String,
// }

// // ======================================================
// // App State
// // ======================================================
// struct AppState {
//     redis: Arc<Mutex<redis::Client>>,
//     mysql: Pool,
//     memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
//     config: AppConfig,
//     shutdown_token: CancellationToken,
//     retry_config: RetryConfig,
// }

// // ======================================================
// // Retry Configuration
// // ======================================================
// #[derive(Clone)]
// struct RetryConfig {
//     max_attempts: u32,
//     initial_delay_ms: u64,
//     max_delay_ms: u64,
//     multiplier: f64,
// }

// impl Default for RetryConfig {
//     fn default() -> Self {
//         Self {
//             max_attempts: 5,
//             initial_delay_ms: 100,
//             max_delay_ms: 5000,
//             multiplier: 2.0,
//         }
//     }
// }

// async fn retry_with_backoff<F, Fut, T, E>(
//     operation: F,
//     config: &RetryConfig,
//     operation_name: &str,
// ) -> Result<T, E>
// where
//     F: Fn() -> Fut,
//     Fut: std::future::Future<Output = Result<T, E>>,
//     E: std::fmt::Display,
// {
//     let mut attempt = 0;
//     let mut delay = config.initial_delay_ms;

//     loop {
//         attempt += 1;

//         match operation().await {
//             Ok(result) => {
//                 if attempt > 1 {
//                     info!(
//                         operation = operation_name,
//                         attempts = attempt,
//                         "Operation succeeded after retry"
//                     );
//                 }
//                 return Ok(result);
//             }
//             Err(e) if attempt >= config.max_attempts => {
//                 error!(
//                     operation = operation_name,
//                     attempts = attempt,
//                     error = %e,
//                     "Operation failed after maximum retry attempts"
//                 );
//                 return Err(e);
//             }
//             Err(e) => {
//                 warn!(
//                     operation = operation_name,
//                     attempt = attempt,
//                     max_attempts = config.max_attempts,
//                     delay_ms = delay,
//                     error = %e,
//                     "Operation failed, retrying..."
//                 );

//                 sleep(Duration::from_millis(delay)).await;
//                 delay = ((delay as f64 * config.multiplier) as u64).min(config.max_delay_ms);
//             }
//         }
//     }
// }

// // ======================================================
// // Business Logic
// // ======================================================
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;
//     score.clamp(0.0, 1.0)
// }

// fn stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ======================================================
// // Sensor Simulation
// // ======================================================
// fn simulate_sensor_data() -> SensorData {
//     let mut rng = rand::thread_rng();
//     SensorData {
//         temperature: rng.gen_range(20.0..35.0),
//         humidity: rng.gen_range(40.0..80.0),
//         noise: rng.gen_range(50.0..90.0),
//         heart_rate: rng.gen_range(60.0..100.0),
//         motion: rng.gen_bool(0.3),
//         timestamp: Utc::now().to_rfc3339(),
//     }
// }

// // ======================================================
// // TCP Serial Reading (Mac/Docker compatible)
// // ======================================================
// async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
//     match TcpStream::connect((host, port)).await {
//         Ok(stream) => {
//             info!(
//                 operation = "tcp_connect",
//                 host = %host,
//                 port = %port,
//                 "Successfully connected to TCP sensor stream"
//             );

//             let mut reader = BufReader::new(stream);
//             let mut line = String::new();
//             match reader.read_line(&mut line).await {
//                 Ok(bytes_read) => {
//                     if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
//                         info!(
//                             operation = "tcp_read",
//                             host = %host,
//                             port = %port,
//                             bytes_read = %bytes_read,
//                             temperature = %sensor.temperature,
//                             heart_rate = %sensor.heart_rate,
//                             "Successfully parsed sensor data from TCP"
//                         );
//                         Some(sensor)
//                     } else {
//                         warn!(
//                             operation = "tcp_parse",
//                             host = %host,
//                             port = %port,
//                             raw_data = %line.trim(),
//                             "Failed to parse JSON from TCP stream"
//                         );
//                         None
//                     }
//                 }
//                 Err(e) => {
//                     error!(
//                         error = %e,
//                         operation = "tcp_read",
//                         host = %host,
//                         port = %port,
//                         "Failed to read data from TCP stream"
//                     );
//                     None
//                 }
//             }
//         }
//         Err(e) => {
//             error!(
//                 error = %e,
//                 operation = "tcp_connect",
//                 host = %host,
//                 port = %port,
//                 "Failed to connect to TCP sensor stream"
//             );
//             None
//         }
//     }
// }

// // ======================================================
// // Background Task with Graceful Shutdown
// // ======================================================
// async fn sensor_task(state: web::Data<AppState>, shutdown_token: CancellationToken) {
//     let mut ticker = interval(Duration::from_secs(1));

//     info!(
//         operation = "sensor_task_start",
//         use_serial = %state.config.use_serial,
//         serial_host = %state.config.serial_tcp_host,
//         serial_port = %state.config.serial_tcp_port,
//         "Sensor background task started"
//     );

//     loop {
//         tokio::select! {
//             _ = shutdown_token.cancelled() => {
//                 info!(
//                     operation = "sensor_task_shutdown",
//                     "Sensor task received shutdown signal, cleaning up..."
//                 );
//                 break;
//             }
//             _ = ticker.tick() => {
//                 if let Err(e) = process_sensor_data(&state).await {
//                     error!(
//                         error = ?e,
//                         operation = "sensor_task_process",
//                         "Error processing sensor data in background task"
//                     );
//                     // Continue running even on errors
//                 }
//             }
//         }
//     }

//     info!(
//         operation = "sensor_task_stopped",
//         "Sensor task stopped gracefully"
//     );
// }

// async fn process_sensor_data(state: &web::Data<AppState>) -> Result<(), ApiError> {
//     let data = if state.config.use_serial {
//         match read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port)
//             .await
//         {
//             Some(sensor_data) => {
//                 info!(
//                     operation = "sensor_data_source",
//                     source = "tcp",
//                     "Using real sensor data from TCP stream"
//                 );
//                 sensor_data
//             }
//             None => {
//                 warn!(
//                     operation = "sensor_data_source",
//                     source = "simulation_fallback",
//                     "TCP read failed, falling back to simulated data"
//                 );
//                 simulate_sensor_data()
//             }
//         }
//     } else {
//         info!(
//             operation = "sensor_data_source",
//             source = "simulation",
//             "Using simulated sensor data"
//         );
//         simulate_sensor_data()
//     };

//     // Validate sensor data
//     if let Err(e) = data.validate() {
//         warn!(
//             operation = "sensor_validation",
//             error = ?e,
//             temperature = %data.temperature,
//             humidity = %data.humidity,
//             heart_rate = %data.heart_rate,
//             "Sensor data validation failed"
//         );
//         return Err(ApiError::Validation(format!("{:?}", e)));
//     }

//     let index = calculate_stress_index(&data);
//     let enhanced = EnhancedSensorData {
//         stress_index: index,
//         stress_level: stress_level(index),
//         data,
//     };

//     // In-memory fallback (always succeeds)
//     {
//         let mut mem = state.memory.lock().await;
//         mem.push_back(enhanced.clone());
//         if mem.len() > 600 {
//             mem.pop_front();
//         }
//         info!(
//             operation = "memory_store",
//             buffer_size = %mem.len(),
//             timestamp = %enhanced.data.timestamp,
//             stress_level = %enhanced.stress_level,
//             "Stored sensor data in memory buffer"
//         );
//     }

//     // Redis (non-blocking with retry)
//     let redis = state.redis.clone();
//     let redis_payload = enhanced.clone();
//     let retry_config = state.retry_config.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_redis(redis, redis_payload, &retry_config).await {
//             warn!(
//                 error = ?e,
//                 operation = "background_redis_store",
//                 "Redis background task failed"
//             );
//         }
//     });

//     // MySQL (non-blocking with retry)
//     let pool = state.mysql.clone();
//     let db_payload = enhanced.clone();
//     let retry_config = state.retry_config.clone();
//     tokio::spawn(async move {
//         if let Err(e) = store_to_mysql(pool, db_payload, &retry_config).await {
//             warn!(
//                 error = ?e,
//                 operation = "background_mysql_store",
//                 "MySQL background task failed"
//             );
//         }
//     });

//     Ok(())
// }

// async fn store_to_redis(
//     redis: Arc<Mutex<redis::Client>>,
//     payload: EnhancedSensorData,
//     retry_config: &RetryConfig,
// ) -> Result<(), ApiError> {
//     let timestamp = payload.data.timestamp.clone();
//     let key = format!("sensor:{}", timestamp);
//     let value = serde_json::to_string(&payload).map_err(|e| {
//         error!(
//             error = %e,
//             operation = "redis_serialization",
//             timestamp = %timestamp,
//             key = %key,
//             "Failed to serialize sensor data for Redis"
//         );
//         ApiError::Redis(format!("Serialization failed: {}", e))
//     })?;

//     // Retry the entire operation (get connection + set)
//     retry_with_backoff(
//         || {
//             let redis = redis.clone();
//             let key = key.clone();
//             let value = value.clone();
//             let timestamp = timestamp.clone();

//             async move {
//                 let mut conn = redis
//                     .lock()
//                     .await
//                     .get_multiplexed_async_connection()
//                     .await
//                     .map_err(|e| {
//                         error!(
//                             error = %e,
//                             operation = "redis_connection",
//                             timestamp = %timestamp,
//                             "Failed to establish Redis connection"
//                         );
//                         ApiError::Redis(format!("Connection failed: {}", e))
//                     })?;

//                 conn.set_ex::<_, _, ()>(key.clone(), value, 600)
//                     .await
//                     .map_err(|e| {
//                         error!(
//                             error = %e,
//                             operation = "redis_set",
//                             timestamp = %timestamp,
//                             key = %key,
//                             ttl = 600,
//                             "Failed to set value in Redis"
//                         );
//                         ApiError::Redis(format!("SET failed: {}", e))
//                     })
//             }
//         },
//         retry_config,
//         "redis_set",
//     )
//     .await?;

//     info!(
//         operation = "redis_set",
//         timestamp = %timestamp,
//         key = %key,
//         stress_level = %payload.stress_level,
//         "Successfully stored sensor data in Redis"
//     );

//     Ok(())
// }

// async fn store_to_mysql(
//     pool: Pool,
//     payload: EnhancedSensorData,
//     retry_config: &RetryConfig,
// ) -> Result<(), ApiError> {
//     let timestamp = payload.data.timestamp.clone();

//     retry_with_backoff(
//         || async {
//             let mut conn = pool
//                 .get_conn()
//                 .await
//                 .map_err(|e| {
//                     error!(
//                         error = %e,
//                         operation = "mysql_connection",
//                         timestamp = %timestamp,
//                         "Failed to get MySQL connection from pool"
//                     );
//                     ApiError::Database(format!("Connection failed: {}", e))
//                 })?;

//             conn.exec_drop(
//                 r#"INSERT INTO sensor_data
//                    (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//                 (
//                     payload.data.temperature,
//                     payload.data.humidity,
//                     payload.data.noise,
//                     payload.data.heart_rate,
//                     payload.data.motion,
//                     payload.stress_index,
//                     payload.stress_level.clone(),
//                     timestamp.clone(),
//                 ),
//             )
//             .await
//             .map_err(|e| {
//                 error!(
//                     error = %e,
//                     operation = "mysql_insert",
//                     timestamp = %timestamp,
//                     temperature = %payload.data.temperature,
//                     humidity = %payload.data.humidity,
//                     heart_rate = %payload.data.heart_rate,
//                     stress_level = %payload.stress_level,
//                     "Failed to insert sensor data into MySQL"
//                 );
//                 ApiError::Database(format!("Insert failed: {}", e))
//             })?;

//             info!(
//                 operation = "mysql_insert",
//                 timestamp = %timestamp,
//                 stress_level = %payload.stress_level,
//                 stress_index = %payload.stress_index,
//                 "Successfully stored sensor data in MySQL"
//             );

//             Ok(())
//         },
//         retry_config,
//         "mysql_insert",
//     )
//     .await
// }

// // ======================================================
// // API
// // ======================================================
// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// // ======================================================
// // Main
// // ======================================================
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .json()
//         .init();

//     let config = AppConfig::from_env_validated().map_err(|e| {
//         error!(error = %e, "Configuration validation failed");
//         std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
//     })?;

//     info!(
//         operation = "application_startup",
//         use_serial = %config.use_serial,
//         bind_addr = %config.bind_addr,
//         serial_tcp_host = %config.serial_tcp_host,
//         serial_tcp_port = %config.serial_tcp_port,
//         "Starting ESMS backend"
//     );

//     // Create cancellation token for graceful shutdown
//     let shutdown_token = CancellationToken::new();

//     // Initialize Redis
//     let redis = redis::Client::open(config.redis_url.clone()).map_err(|e| {
//         error!(
//             error = %e,
//             operation = "redis_init",
//             "Failed to initialize Redis client"
//         );
//         std::io::Error::new(
//             std::io::ErrorKind::Other,
//             format!("Redis init failed: {}", e),
//         )
//     })?;

//     info!(
//         operation = "redis_initialized",
//         "Redis client initialized successfully"
//     );

//     // Initialize MySQL
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     info!(
//         operation = "mysql_initialized",
//         "MySQL connection pool initialized successfully"
//     );

//     // Create app state
//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
//         config: config.clone(),
//         shutdown_token: shutdown_token.clone(),
//         retry_config: RetryConfig::default(),
//     });

//     // Spawn background sensor task with shutdown token
//     let sensor_task_handle = tokio::spawn(sensor_task(state.clone(), shutdown_token.child_token()));

//     // Create HTTP server
//     let server = HttpServer::new(move || {
//         App::new()
//             .wrap(Logger::default())
//             .wrap(Cors::permissive())
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//     })
//     .bind(&config.bind_addr)?
//     .run();

//     info!(
//         operation = "http_server_started",
//         bind_addr = %config.bind_addr,
//         "HTTP server is running"
//     );

//     let server_handle = server.handle();

//     // Setup graceful shutdown signal handler
//     let shutdown_signal = async move {
//         tokio::signal::ctrl_c()
//             .await
//             .expect("Failed to listen for ctrl-c");

//         info!(
//             operation = "shutdown_signal_received",
//             "Shutdown signal received, initiating graceful shutdown..."
//         );

//         // Trigger cancellation token to stop background tasks
//         shutdown_token.cancel();

//         // Stop HTTP server gracefully
//         server_handle.stop(true).await;

//         info!(operation = "http_server_stopped", "HTTP server stopped");
//     };

//     // Run server and wait for shutdown signal
//     tokio::select! {
//         result = server => {
//             result?;
//         }
//         _ = shutdown_signal => {
//             info!(
//                 operation = "shutdown_signal_handled",
//                 "Shutdown signal handled"
//             );
//         }
//     }

//     // Wait for background task to complete
//     match tokio::time::timeout(Duration::from_secs(10), sensor_task_handle).await {
//         Ok(Ok(())) => info!(
//             operation = "background_task_stopped",
//             "Background task stopped successfully"
//         ),
//         Ok(Err(e)) => error!(
//             error = ?e,
//             operation = "background_task_error",
//             "Background task encountered an error during shutdown"
//         ),
//         Err(_) => error!(
//             operation = "background_task_timeout",
//             timeout_seconds = 10,
//             "Background task did not stop within timeout"
//         ),
//     }

//     info!(
//         operation = "application_shutdown_complete",
//         "Application shutdown complete"
//     );
//     Ok(())
// }

// // ======================================================
// // Unit Tests
// // ======================================================
// #[cfg(test)]
// mod tests {
//     use super::*;

//     // ====================================================
//     // Stress Calculation Tests
//     // ====================================================

//     #[test]
//     fn test_stress_index_returns_value_in_range() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 75.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         let index = calculate_stress_index(&data);
//         assert!(
//             index >= 0.0 && index <= 1.0,
//             "Stress index should be between 0 and 1"
//         );
//     }

//     #[test]
//     fn test_stress_index_minimum_values() {
//         let data = SensorData {
//             temperature: 0.0,
//             humidity: 0.0,
//             noise: 0.0,
//             heart_rate: 60.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         let index = calculate_stress_index(&data);
//         assert!(index < 0.1, "Minimum stress should be very low");
//     }

//     #[test]
//     fn test_stress_index_maximum_values() {
//         let data = SensorData {
//             temperature: 50.0,
//             humidity: 100.0,
//             noise: 100.0,
//             heart_rate: 160.0,
//             motion: true,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         let index = calculate_stress_index(&data);
//         assert!(index > 0.5, "High vitals should produce high stress index");
//     }

//     #[test]
//     fn test_stress_index_moderate_values() {
//         let data = SensorData {
//             temperature: 30.0,
//             humidity: 60.0,
//             noise: 70.0,
//             heart_rate: 90.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         let index = calculate_stress_index(&data);
//         assert!(
//             index >= 0.3 && index <= 0.7,
//             "Moderate values should produce moderate stress"
//         );
//     }

//     // ====================================================
//     // Stress Level Classification Tests
//     // ====================================================

//     #[test]
//     fn test_stress_level_low() {
//         assert_eq!(stress_level(0.0), "Low");
//         assert_eq!(stress_level(0.1), "Low");
//         assert_eq!(stress_level(0.29), "Low");
//     }

//     #[test]
//     fn test_stress_level_moderate() {
//         assert_eq!(stress_level(0.3), "Moderate");
//         assert_eq!(stress_level(0.45), "Moderate");
//         assert_eq!(stress_level(0.59), "Moderate");
//     }

//     #[test]
//     fn test_stress_level_high() {
//         assert_eq!(stress_level(0.6), "High");
//         assert_eq!(stress_level(0.8), "High");
//         assert_eq!(stress_level(1.0), "High");
//     }

//     #[test]
//     fn test_stress_level_boundary_conditions() {
//         assert_eq!(stress_level(0.29999), "Low");
//         assert_eq!(stress_level(0.30000), "Moderate");
//         assert_eq!(stress_level(0.59999), "Moderate");
//         assert_eq!(stress_level(0.60000), "High");
//     }

//     // ====================================================
//     // Sensor Data Validation Tests
//     // ====================================================

//     #[test]
//     fn test_sensor_data_valid() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 75.0,
//             motion: true,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_ok(),
//             "Valid sensor data should pass validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_temperature_too_low() {
//         let data = SensorData {
//             temperature: -5.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 75.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Temperature below 0 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_temperature_too_high() {
//         let data = SensorData {
//             temperature: 65.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 75.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Temperature above 60 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_humidity() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 105.0,
//             noise: 70.0,
//             heart_rate: 75.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Humidity above 100 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_noise() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 125.0,
//             heart_rate: 75.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Noise above 120 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_heart_rate_too_low() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 25.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Heart rate below 30 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_invalid_heart_rate_too_high() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 70.0,
//             heart_rate: 205.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };

//         assert!(
//             data.validate().is_err(),
//             "Heart rate above 200 should fail validation"
//         );
//     }

//     #[test]
//     fn test_sensor_data_boundary_values() {
//         let min_data = SensorData {
//             temperature: 0.0,
//             humidity: 0.0,
//             noise: 0.0,
//             heart_rate: 30.0,
//             motion: false,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };
//         assert!(
//             min_data.validate().is_ok(),
//             "Minimum boundary values should be valid"
//         );

//         let max_data = SensorData {
//             temperature: 60.0,
//             humidity: 100.0,
//             noise: 120.0,
//             heart_rate: 200.0,
//             motion: true,
//             timestamp: "2024-01-01T00:00:00Z".to_string(),
//         };
//         assert!(
//             max_data.validate().is_ok(),
//             "Maximum boundary values should be valid"
//         );
//     }

//     // ====================================================
//     // Simulation Tests
//     // ====================================================

//     #[test]
//     fn test_simulate_sensor_data_returns_valid_data() {
//         for _ in 0..100 {
//             let data = simulate_sensor_data();
//             assert!(
//                 data.validate().is_ok(),
//                 "Simulated data should always be valid"
//             );
//         }
//     }

//     #[test]
//     fn test_simulate_sensor_data_ranges() {
//         for _ in 0..100 {
//             let data = simulate_sensor_data();

//             assert!(data.temperature >= 20.0 && data.temperature < 35.0);
//             assert!(data.humidity >= 40.0 && data.humidity < 80.0);
//             assert!(data.noise >= 50.0 && data.noise < 90.0);
//             assert!(data.heart_rate >= 60.0 && data.heart_rate < 100.0);
//         }
//     }

//     #[test]
//     fn test_simulate_sensor_data_has_timestamp() {
//         let data = simulate_sensor_data();
//         assert!(!data.timestamp.is_empty(), "Timestamp should not be empty");
//     }
// }

// v2

#![allow(clippy::multiple_crate_versions)]

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use chrono::Utc;
use dotenv::dotenv;
use mysql_async::{prelude::Queryable, Opts, Pool};
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, env, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::Mutex,
    time::{interval, sleep, Duration},
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};
use url::Url;
use validator::Validate;

// ======================================================
// Configuration
// ======================================================
#[derive(Clone)]
struct AppConfig {
    redis_url: String,
    mysql_url: String,
    bind_addr: String,
    use_serial: bool,
    serial_tcp_host: String,
    serial_tcp_port: u16,
}

impl AppConfig {
    fn from_env() -> Self {
        let use_serial = env::var("USE_SERIAL")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let serial_tcp_port = env::var("SERIAL_TCP_PORT")
            .unwrap_or_else(|_| "5555".to_string())
            .parse::<u16>()
            .unwrap_or(5555);

        Self {
            redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
            mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            use_serial,
            serial_tcp_host: env::var("SERIAL_TCP_HOST")
                .unwrap_or_else(|_| "host.docker.internal".to_string()),
            serial_tcp_port,
        }
    }

    pub fn from_env_validated() -> Result<Self, ConfigError> {
        let config = Self::from_env();
        validate_config(&config)?;
        Ok(config)
    }
}

// ======================================================
// Error Handling (Centralized)
// ======================================================
#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
enum ApiError {
    #[error("Internal server error")]
    Internal,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("TCP connection error: {0}")]
    TcpConnection(String),
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Internal => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "An internal error occurred"
            })),
            ApiError::Database(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "database_error",
                    "message": msg
                }))
            }
            ApiError::Redis(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "redis_error",
                "message": msg
            })),
            ApiError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })),
            ApiError::TcpConnection(msg) => {
                HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": "tcp_connection_error",
                    "message": msg
                }))
            }
        }
    }
}

// ======================================================
// Configuration Error
// ======================================================
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Invalid Redis URL: {0}")]
    InvalidRedisUrl(String),

    #[error("Invalid MySQL URL: {0}")]
    InvalidMysqlUrl(String),

    #[error("Invalid bind address: {0}")]
    InvalidBindAddr(String),

    #[error("Invalid serial TCP configuration: {0}")]
    InvalidSerialConfig(String),
}

// ======================================================
// Configuration Validation
// ======================================================
fn validate_config(config: &AppConfig) -> Result<(), ConfigError> {
    info!(
        operation = "config_validation_start",
        "Starting configuration validation"
    );

    validate_redis_url(&config.redis_url)?;
    validate_mysql_url(&config.mysql_url)?;
    validate_bind_addr(&config.bind_addr)?;

    if config.use_serial {
        validate_serial_config(&config.serial_tcp_host, config.serial_tcp_port)?;
    }

    info!(
        operation = "config_validation_complete",
        "Configuration validation successful"
    );

    Ok(())
}

fn validate_redis_url(url: &str) -> Result<(), ConfigError> {
    if url.is_empty() {
        return Err(ConfigError::InvalidRedisUrl(
            "Redis URL is empty".to_string(),
        ));
    }

    if !url.starts_with("redis://") && !url.starts_with("rediss://") {
        return Err(ConfigError::InvalidRedisUrl(format!(
            "Redis URL must start with redis:// or rediss://, got: {url}"
        )));
    }

    info!(
        operation = "config_validation",
        component = "redis_url",
        "Redis URL validated successfully"
    );

    Ok(())
}

fn validate_mysql_url(url: &str) -> Result<(), ConfigError> {
    if url.is_empty() {
        return Err(ConfigError::InvalidMysqlUrl(
            "MySQL URL is empty".to_string(),
        ));
    }

    if !url.starts_with("mysql://") {
        return Err(ConfigError::InvalidMysqlUrl(format!(
            "MySQL URL must start with mysql://, got: {url}"
        )));
    }

    Url::parse(url)
        .map_err(|e| ConfigError::InvalidMysqlUrl(format!("Invalid URL format: {e}")))?;

    info!(
        operation = "config_validation",
        component = "mysql_url",
        "MySQL URL validated successfully"
    );

    Ok(())
}

fn validate_bind_addr(addr: &str) -> Result<(), ConfigError> {
    if addr.is_empty() {
        return Err(ConfigError::InvalidBindAddr(
            "Bind address is empty".to_string(),
        ));
    }

    addr.parse::<SocketAddr>()
        .map_err(|e| ConfigError::InvalidBindAddr(format!("Invalid socket address format: {e}")))?;

    info!(
        operation = "config_validation",
        component = "bind_addr",
        bind_addr = %addr,
        "Bind address validated successfully"
    );

    Ok(())
}

fn validate_serial_config(host: &str, port: u16) -> Result<(), ConfigError> {
    if host.is_empty() {
        return Err(ConfigError::InvalidSerialConfig(
            "Serial TCP host is empty".to_string(),
        ));
    }

    if port == 0 {
        return Err(ConfigError::InvalidSerialConfig(format!(
            "Serial TCP port {port} is out of valid range (1-65535)"
        )));
    }

    info!(
        operation = "config_validation",
        component = "serial_tcp",
        host = %host,
        port = %port,
        "Serial TCP configuration validated successfully"
    );

    Ok(())
}

// ======================================================
// Models
// ======================================================
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
struct SensorData {
    #[validate(range(min = 0.0, max = 60.0))]
    temperature: f64,
    #[validate(range(min = 0.0, max = 100.0))]
    humidity: f64,
    #[validate(range(min = 0.0, max = 120.0))]
    noise: f64,
    #[validate(range(min = 30.0, max = 200.0))]
    heart_rate: f64,
    motion: bool,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
struct EnhancedSensorData {
    #[serde(flatten)]
    data: SensorData,
    stress_index: f64,
    stress_level: String,
}

// ======================================================
// App State
// ======================================================
struct AppState {
    redis: Arc<Mutex<redis::Client>>,
    mysql: Pool,
    memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    config: AppConfig,
    #[allow(dead_code)]
    shutdown_token: CancellationToken,
    retry_config: RetryConfig,
}

// ======================================================
// Retry Configuration
// ======================================================
#[derive(Clone)]
struct RetryConfig {
    max_attempts: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
    multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        operation = operation_name,
                        attempts = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) if attempt >= config.max_attempts => {
                error!(
                    operation = operation_name,
                    attempts = attempt,
                    error = %e,
                    "Operation failed after maximum retry attempts"
                );
                return Err(e);
            }
            Err(e) => {
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    delay_ms = delay,
                    error = %e,
                    "Operation failed, retrying..."
                );

                sleep(Duration::from_millis(delay)).await;
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                {
                    delay = ((delay as f64 * config.multiplier) as u64).min(config.max_delay_ms);
                }
            }
        }
    }
}

// ======================================================
// Business Logic
// ======================================================
fn calculate_stress_index(data: &SensorData) -> f64 {
    let score = (data.heart_rate - 60.0) / 100.0 * 0.5
        + (data.temperature / 50.0) * 0.2
        + (data.humidity / 100.0) * 0.2
        + (data.noise / 100.0) * 0.1;
    score.clamp(0.0, 1.0)
}

fn stress_level(score: f64) -> String {
    match score {
        x if x < 0.3 => "Low",
        x if x < 0.6 => "Moderate",
        _ => "High",
    }
    .to_string()
}

// ======================================================
// Sensor Simulation
// ======================================================
fn simulate_sensor_data() -> SensorData {
    let mut rng = rand::thread_rng();
    SensorData {
        temperature: rng.gen_range(20.0..35.0),
        humidity: rng.gen_range(40.0..80.0),
        noise: rng.gen_range(50.0..90.0),
        heart_rate: rng.gen_range(60.0..100.0),
        motion: rng.gen_bool(0.3),
        timestamp: Utc::now().to_rfc3339(),
    }
}

// ======================================================
// TCP Serial Reading (Mac/Docker compatible)
// ======================================================
async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
    match TcpStream::connect((host, port)).await {
        Ok(stream) => {
            info!(
                operation = "tcp_connect",
                host = %host,
                port = %port,
                "Successfully connected to TCP sensor stream"
            );

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(bytes_read) => {
                    if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
                        info!(
                            operation = "tcp_read",
                            host = %host,
                            port = %port,
                            bytes_read = %bytes_read,
                            temperature = %sensor.temperature,
                            heart_rate = %sensor.heart_rate,
                            "Successfully parsed sensor data from TCP"
                        );
                        Some(sensor)
                    } else {
                        warn!(
                            operation = "tcp_parse",
                            host = %host,
                            port = %port,
                            raw_data = %line.trim(),
                            "Failed to parse JSON from TCP stream"
                        );
                        None
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        operation = "tcp_read",
                        host = %host,
                        port = %port,
                        "Failed to read data from TCP stream"
                    );
                    None
                }
            }
        }
        Err(e) => {
            error!(
                error = %e,
                operation = "tcp_connect",
                host = %host,
                port = %port,
                "Failed to connect to TCP sensor stream"
            );
            None
        }
    }
}

// ======================================================
// Background Task with Graceful Shutdown
// ======================================================
async fn sensor_task(state: web::Data<AppState>, shutdown_token: CancellationToken) {
    let mut ticker = interval(Duration::from_secs(1));

    info!(
        operation = "sensor_task_start",
        use_serial = %state.config.use_serial,
        serial_host = %state.config.serial_tcp_host,
        serial_port = %state.config.serial_tcp_port,
        "Sensor background task started"
    );

    loop {
        tokio::select! {
            () = shutdown_token.cancelled() => {
                info!(
                    operation = "sensor_task_shutdown",
                    "Sensor task received shutdown signal, cleaning up..."
                );
                break;
            }
            _ = ticker.tick() => {
                if let Err(e) = process_sensor_data(&state).await {
                    error!(
                        error = ?e,
                        operation = "sensor_task_process",
                        "Error processing sensor data in background task"
                    );
                    // Continue running even on errors
                }
            }
        }
    }

    info!(
        operation = "sensor_task_stopped",
        "Sensor task stopped gracefully"
    );
}

async fn process_sensor_data(state: &web::Data<AppState>) -> Result<(), ApiError> {
    let data = if state.config.use_serial {
        if let Some(sensor_data) =
            read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port).await
        {
            info!(
                operation = "sensor_data_source",
                source = "tcp",
                "Using real sensor data from TCP stream"
            );
            sensor_data
        } else {
            warn!(
                operation = "sensor_data_source",
                source = "simulation_fallback",
                "TCP read failed, falling back to simulated data"
            );
            simulate_sensor_data()
        }
    } else {
        info!(
            operation = "sensor_data_source",
            source = "simulation",
            "Using simulated sensor data"
        );
        simulate_sensor_data()
    };

    // Validate sensor data
    if let Err(e) = data.validate() {
        warn!(
            operation = "sensor_validation",
            error = ?e,
            temperature = %data.temperature,
            humidity = %data.humidity,
            heart_rate = %data.heart_rate,
            "Sensor data validation failed"
        );
        return Err(ApiError::Validation(format!("{e:?}")));
    }

    let index = calculate_stress_index(&data);
    let enhanced = EnhancedSensorData {
        stress_index: index,
        stress_level: stress_level(index),
        data,
    };

    // In-memory fallback (always succeeds)
    {
        let mut mem = state.memory.lock().await;
        mem.push_back(enhanced.clone());
        if mem.len() > 600 {
            mem.pop_front();
        }
        info!(
            operation = "memory_store",
            buffer_size = %mem.len(),
            timestamp = %enhanced.data.timestamp,
            stress_level = %enhanced.stress_level,
            "Stored sensor data in memory buffer"
        );
    }

    // Redis (non-blocking with retry)
    let redis = state.redis.clone();
    let redis_payload = enhanced.clone();
    let retry_config = state.retry_config.clone();
    tokio::spawn(async move {
        if let Err(e) = store_to_redis(redis, redis_payload, &retry_config).await {
            warn!(
                error = ?e,
                operation = "background_redis_store",
                "Redis background task failed"
            );
        }
    });

    // MySQL (non-blocking with retry)
    let pool = state.mysql.clone();
    let db_payload = enhanced.clone();
    let retry_config = state.retry_config.clone();
    tokio::spawn(async move {
        if let Err(e) = store_to_mysql(pool, db_payload, &retry_config).await {
            warn!(
                error = ?e,
                operation = "background_mysql_store",
                "MySQL background task failed"
            );
        }
    });

    Ok(())
}

async fn store_to_redis(
    redis: Arc<Mutex<redis::Client>>,
    payload: EnhancedSensorData,
    retry_config: &RetryConfig,
) -> Result<(), ApiError> {
    let timestamp = payload.data.timestamp.clone();
    let key = format!("sensor:{timestamp}");
    let value = serde_json::to_string(&payload).map_err(|e| {
        error!(
            error = %e,
            operation = "redis_serialization",
            timestamp = %timestamp,
            key = %key,
            "Failed to serialize sensor data for Redis"
        );
        ApiError::Redis(format!("Serialization failed: {e}"))
    })?;

    // Retry the entire operation (get connection + set)
    retry_with_backoff(
        || {
            let redis = redis.clone();
            let key = key.clone();
            let value = value.clone();
            let timestamp = timestamp.clone();

            async move {
                let mut conn = redis
                    .lock()
                    .await
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            operation = "redis_connection",
                            timestamp = %timestamp,
                            "Failed to establish Redis connection"
                        );
                        ApiError::Redis(format!("Connection failed: {e}"))
                    })?;

                conn.set_ex::<_, _, ()>(key.clone(), value, 600)
                    .await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            operation = "redis_set",
                            timestamp = %timestamp,
                            key = %key,
                            ttl = 600,
                            "Failed to set value in Redis"
                        );
                        ApiError::Redis(format!("SET failed: {e}"))
                    })
            }
        },
        retry_config,
        "redis_set",
    )
    .await?;

    info!(
        operation = "redis_set",
        timestamp = %timestamp,
        key = %key,
        stress_level = %payload.stress_level,
        "Successfully stored sensor data in Redis"
    );

    Ok(())
}

async fn store_to_mysql(
    pool: Pool,
    payload: EnhancedSensorData,
    retry_config: &RetryConfig,
) -> Result<(), ApiError> {
    let timestamp = payload.data.timestamp.clone();

    retry_with_backoff(
        || async {
            let mut conn = pool
                .get_conn()
                .await
                .map_err(|e| {
                    error!(
                        error = %e,
                        operation = "mysql_connection",
                        timestamp = %timestamp,
                        "Failed to get MySQL connection from pool"
                    );
                    ApiError::Database(format!("Connection failed: {e}"))
                })?;

            conn.exec_drop(
                r"INSERT INTO sensor_data 
                   (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp) 
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (
                    payload.data.temperature,
                    payload.data.humidity,
                    payload.data.noise,
                    payload.data.heart_rate,
                    payload.data.motion,
                    payload.stress_index,
                    payload.stress_level.clone(),
                    timestamp.clone(),
                ),
            )
            .await
            .map_err(|e| {
                error!(
                    error = %e,
                    operation = "mysql_insert",
                    timestamp = %timestamp,
                    temperature = %payload.data.temperature,
                    humidity = %payload.data.humidity,
                    heart_rate = %payload.data.heart_rate,
                    stress_level = %payload.stress_level,
                    "Failed to insert sensor data into MySQL"
                );
                ApiError::Database(format!("Insert failed: {e}"))
            })?;

            info!(
                operation = "mysql_insert",
                timestamp = %timestamp,
                stress_level = %payload.stress_level,
                stress_index = %payload.stress_index,
                "Successfully stored sensor data in MySQL"
            );

            Ok(())
        },
        retry_config,
        "mysql_insert",
    )
    .await
}

// ======================================================
// API
// ======================================================
async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    })))
}

async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.memory.lock().await;
    let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
    Ok(HttpResponse::Ok().json(data))
}

// ======================================================
// Main
// ======================================================
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

// ======================================================
// Unit Tests
// ======================================================
#[cfg(test)]
mod tests {
    use super::*;

    // ====================================================
    // Stress Calculation Tests
    // ====================================================

    #[test]
    fn test_stress_index_returns_value_in_range() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(
            (0.0..=1.0).contains(&index),
            "Stress index should be between 0 and 1"
        );
    }

    #[test]
    fn test_stress_index_minimum_values() {
        let data = SensorData {
            temperature: 0.0,
            humidity: 0.0,
            noise: 0.0,
            heart_rate: 60.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(index < 0.1, "Minimum stress should be very low");
    }

    #[test]
    fn test_stress_index_maximum_values() {
        let data = SensorData {
            temperature: 50.0,
            humidity: 100.0,
            noise: 100.0,
            heart_rate: 160.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(index > 0.5, "High vitals should produce high stress index");
    }

    #[test]
    fn test_stress_index_moderate_values() {
        let data = SensorData {
            temperature: 30.0,
            humidity: 60.0,
            noise: 70.0,
            heart_rate: 90.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(
            (0.3..=0.7).contains(&index),
            "Moderate values should produce moderate stress"
        );
    }

    // ====================================================
    // Stress Level Classification Tests
    // ====================================================

    #[test]
    fn test_stress_level_low() {
        assert_eq!(stress_level(0.0), "Low");
        assert_eq!(stress_level(0.1), "Low");
        assert_eq!(stress_level(0.29), "Low");
    }

    #[test]
    fn test_stress_level_moderate() {
        assert_eq!(stress_level(0.3), "Moderate");
        assert_eq!(stress_level(0.45), "Moderate");
        assert_eq!(stress_level(0.59), "Moderate");
    }

    #[test]
    fn test_stress_level_high() {
        assert_eq!(stress_level(0.6), "High");
        assert_eq!(stress_level(0.8), "High");
        assert_eq!(stress_level(1.0), "High");
    }

    #[test]
    fn test_stress_level_boundary_conditions() {
        assert_eq!(stress_level(0.29999), "Low");
        assert_eq!(stress_level(0.30000), "Moderate");
        assert_eq!(stress_level(0.59999), "Moderate");
        assert_eq!(stress_level(0.60000), "High");
    }

    // ====================================================
    // Sensor Data Validation Tests
    // ====================================================

    #[test]
    fn test_sensor_data_valid() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Valid sensor data should pass validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_temperature_too_low() {
        let data = SensorData {
            temperature: -5.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Temperature below 0 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_temperature_too_high() {
        let data = SensorData {
            temperature: 65.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Temperature above 60 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_humidity() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 105.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Humidity above 100 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_noise() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 125.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Noise above 120 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_heart_rate_too_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 25.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate below 30 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_heart_rate_too_high() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 205.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate above 200 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_boundary_values() {
        let min_data = SensorData {
            temperature: 0.0,
            humidity: 0.0,
            noise: 0.0,
            heart_rate: 30.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(
            min_data.validate().is_ok(),
            "Minimum boundary values should be valid"
        );

        let max_data = SensorData {
            temperature: 60.0,
            humidity: 100.0,
            noise: 120.0,
            heart_rate: 200.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(
            max_data.validate().is_ok(),
            "Maximum boundary values should be valid"
        );
    }

    // ====================================================
    // Simulation Tests
    // ====================================================

    #[test]
    fn test_simulate_sensor_data_returns_valid_data() {
        for _ in 0..100 {
            let data = simulate_sensor_data();
            assert!(
                data.validate().is_ok(),
                "Simulated data should always be valid"
            );
        }
    }

    #[test]
    fn test_simulate_sensor_data_ranges() {
        for _ in 0..100 {
            let data = simulate_sensor_data();

            assert!(data.temperature >= 20.0 && data.temperature < 35.0);
            assert!(data.humidity >= 40.0 && data.humidity < 80.0);
            assert!(data.noise >= 50.0 && data.noise < 90.0);
            assert!(data.heart_rate >= 60.0 && data.heart_rate < 100.0);
        }
    }

    #[test]
    fn test_simulate_sensor_data_has_timestamp() {
        let data = simulate_sensor_data();
        assert!(!data.timestamp.is_empty(), "Timestamp should not be empty");
    }
}
