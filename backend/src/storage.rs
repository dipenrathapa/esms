use mysql_async::{prelude::Queryable, Pool};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::error::ApiError;
use crate::models::EnhancedSensorData;
use crate::retry::{retry_with_backoff, RetryConfig};

/// Stores the given sensor data payload into Redis with a TTL of 600 seconds.
/// Retries the operation based on the provided retry configuration.
/// # Arguments
/// * `redis` - An Arc-wrapped Mutex containing the Redis client.
/// * `payload` - The sensor data payload to store.
/// * `retry_config` - Configuration for retrying the operation.
/// # Returns
/// * `Ok(())` if the operation succeeds.
/// * `Err(ApiError)` if the operation fails after all retry attempts.
pub async fn store_to_redis(
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

/// Stores the given sensor data payload into MySQL database.
/// Retries the operation based on the provided retry configuration.
/// # Arguments
/// * `pool` - The MySQL connection pool.
/// * `payload` - The sensor data payload to store.                 
/// * `retry_config` - Configuration for retrying the operation.
/// # Returns
/// * `Ok(())` if the operation succeeds.
/// * `Err(ApiError)` if the operation fails after all retry attempts.
pub async fn store_to_mysql(
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
