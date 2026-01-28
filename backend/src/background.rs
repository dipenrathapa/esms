use actix_web::web;
use tokio::time::{interval, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use validator::Validate;

use crate::business::{calculate_stress_index, stress_level};
use crate::error::ApiError;
use crate::models::EnhancedSensorData;
use crate::sensor::{read_sensor_from_tcp, simulate_sensor_data};
use crate::state::AppState;
use crate::storage::{store_to_mysql, store_to_redis};

pub async fn sensor_task(state: web::Data<AppState>, shutdown_token: CancellationToken) {
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
