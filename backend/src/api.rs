// use actix_web::{web, HttpResponse, Result};
// use chrono::{DateTime, NaiveDateTime, Utc};
// use mysql_async::prelude::Queryable;
// use redis::AsyncCommands;
// use serde::Deserialize;
// use tracing::{error, info, warn};

// use crate::error::ApiError;
// use crate::models::EnhancedSensorData;
// use crate::state::AppState;

// pub async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// pub async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// pub async fn get_redis_history(state: web::Data<AppState>) -> Result<HttpResponse> {
//     info!(
//         operation = "redis_history_request",
//         "Fetching last 60 seconds of data from Redis"
//     );

//     let mut conn = state
//         .redis
//         .lock()
//         .await
//         .get_multiplexed_async_connection()
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_connection",
//                 "Failed to establish Redis connection for history"
//             );
//             ApiError::Redis(format!("Connection failed: {e}"))
//         })?;

//     let keys: Vec<String> = conn.keys("sensor:*").await.map_err(|e| {
//         error!(
//             error = %e,
//             operation = "redis_keys",
//             "Failed to retrieve keys from Redis"
//         );
//         ApiError::Redis(format!("Failed to retrieve keys: {e}"))
//     })?;

//     if keys.is_empty() {
//         info!(operation = "redis_history_empty", "No data found in Redis");
//         return Ok(HttpResponse::Ok().json(Vec::<EnhancedSensorData>::new()));
//     }

//     let mut results: Vec<EnhancedSensorData> = Vec::new();

//     for key in keys {
//         match conn.get::<_, Option<String>>(&key).await {
//             Ok(Some(value)) => match serde_json::from_str::<EnhancedSensorData>(&value) {
//                 Ok(data) => results.push(data),
//                 Err(e) => warn!(
//                     error = %e,
//                     key = %key,
//                     operation = "redis_deserialize",
//                     "Failed to deserialize data from Redis"
//                 ),
//             },
//             Ok(None) => warn!(
//                 key = %key,
//                 operation = "redis_get",
//                 "Key returned no value"
//             ),
//             Err(e) => warn!(
//                 error = %e,
//                 key = %key,
//                 operation = "redis_get",
//                 "Failed to get value from Redis"
//             ),
//         }
//     }

//     results.sort_by(|a, b| b.data.timestamp.cmp(&a.data.timestamp));
//     let last_60: Vec<_> = results.into_iter().take(60).collect();

//     Ok(HttpResponse::Ok().json(last_60))
// }

// #[derive(Deserialize)]
// pub struct HistoryQuery {
//     start: String,
//     end: String,
// }

// /// Parse timestamp that could be in either RFC3339 or `MySQL` datetime format
// fn parse_flexible_timestamp(timestamp: &str) -> Result<DateTime<Utc>, String> {
//     if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
//         return Ok(dt.to_utc());
//     }

//     if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
//         return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
//     }

//     if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z") {
//         return Ok(dt.to_utc());
//     }

//     Err(format!("Unable to parse timestamp: {timestamp}"))
// }

// /// Convert `MySQL` datetime format to RFC3339 for consistency
// fn format_timestamp_rfc3339(timestamp: &str) -> String {
//     match parse_flexible_timestamp(timestamp) {
//         Ok(dt) => dt.to_rfc3339(),
//         Err(_) => timestamp.to_string(),
//     }
// }

// #[allow(clippy::too_many_lines)]
// #[allow(clippy::type_complexity)]
// pub async fn get_history(
//     state: web::Data<AppState>,
//     query: web::Query<HistoryQuery>,
// ) -> Result<HttpResponse> {
//     let start_time = parse_flexible_timestamp(&query.start)
//         .map_err(|e| ApiError::Validation(format!("Invalid start timestamp: {e}")))?;

//     let end_time = parse_flexible_timestamp(&query.end)
//         .map_err(|e| ApiError::Validation(format!("Invalid end timestamp: {e}")))?;

//     if start_time >= end_time {
//         return Err(ApiError::Validation("Start time must be before end time".to_string()).into());
//     }

//     let start_mysql = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
//     let end_mysql = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

//     let mut conn = state
//         .mysql
//         .get_conn()
//         .await
//         .map_err(|e| ApiError::Database(format!("Connection failed: {e}")))?;

//     let results: Vec<(f64, f64, f64, f64, bool, f64, String, String)> = conn
//         .exec(
//             r"SELECT temperature, humidity, noise, heart_rate, motion, stress_index, stress_level,
//               CAST(timestamp AS CHAR) as timestamp
//               FROM sensor_data
//               WHERE timestamp >= ? AND timestamp <= ?
//               ORDER BY timestamp DESC",
//             (start_mysql, end_mysql),
//         )
//         .await
//         .map_err(|e| ApiError::Database(format!("Query failed: {e}")))?;

//     let data = results
//         .into_iter()
//         .map(
//             |(
//                 temperature,
//                 humidity,
//                 noise,
//                 heart_rate,
//                 motion,
//                 stress_index,
//                 stress_level,
//                 timestamp,
//             )| EnhancedSensorData {
//                 data: crate::models::SensorData {
//                     temperature,
//                     humidity,
//                     noise,
//                     heart_rate,
//                     motion,
//                     timestamp: format_timestamp_rfc3339(&timestamp),
//                 },
//                 stress_index,
//                 stress_level,
//             },
//         )
//         .collect::<Vec<_>>();

//     Ok(HttpResponse::Ok().json(data))
// }

// #[allow(clippy::too_many_lines)]
// #[allow(clippy::manual_let_else)]
// #[allow(clippy::single_match_else)]
// #[allow(clippy::match_same_arms)]
// pub async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let latest = mem.back().cloned();
//     drop(mem);

//     let data = match latest {
//         Some(d) => d,
//         None => {
//             return Ok(HttpResponse::NotFound().json(serde_json::json!({
//                 "error": "no_data",
//                 "message": "No sensor data available"
//             })));
//         }
//     };

//     let fhir_response = serde_json::json!({
//         "resourceType": "Observation",
//         "id": format!("stress-{}", data.data.timestamp),
//         "status": "final",
//         "effectiveDateTime": data.data.timestamp,
//         "issued": Utc::now().to_rfc3339(),
//         "valueQuantity": {
//             "value": data.stress_index
//         },
//         "interpretation": [{
//             "coding": [{
//                 "code": match data.stress_level.as_str() {
//                     "Low" => "L",
//                     "Moderate" => "N",
//                     "High" => "H",
//                     _ => "N"
//                 }
//             }]
//         }]
//     });

//     Ok(HttpResponse::Ok().json(fhir_response))
// }



use actix_web::{web, HttpResponse, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use mysql_async::prelude::Queryable;
use redis::AsyncCommands;
use serde::Deserialize;
use tracing::{error, info, warn};

use crate::error::ApiError;
use crate::models::EnhancedSensorData;
use crate::state::AppState;

pub async fn health(state: web::Data<AppState>) -> Result<HttpResponse> {
    // Backend is OK if we reached here
    let backend_status = "ok";

    // ---------- MySQL health ----------
    let mysql_status = match state.mysql.get_conn().await {
        Ok(mut conn) => {
            match conn.query_drop("SELECT 1").await {
                Ok(_) => "ok",
                Err(e) => {
                    warn!(
                        error = %e,
                        operation = "health_mysql_query",
                        "MySQL reachable but query failed"
                    );
                    "degraded"
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                operation = "health_mysql_connect",
                "MySQL connection failed"
            );
            "down"
        }
    };

    // ---------- Redis health ----------
    let redis_status = match state
        .redis
        .lock()
        .await
        .get_multiplexed_async_connection()
        .await
    {
        Ok(mut conn) => {
            let ping: redis::RedisResult<String> =
                redis::cmd("PING").query_async(&mut conn).await;

            match ping {
                Ok(_) => "ok",
                Err(e) => {
                    warn!(
                        error = %e,
                        operation = "health_redis_ping",
                        "Redis ping failed"
                    );
                    "degraded"
                }
            }
        }
        Err(e) => {
            warn!(
                error = %e,
                operation = "health_redis_connect",
                "Redis connection failed"
            );
            "down"
        }
    };

    // ---------- System mode ----------
    let mode = if state.config.use_serial {
        "serial_sensor"
    } else {
        "simulated_sensor"
    };

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "backend": backend_status,
        "mysql": mysql_status,
        "redis": redis_status,
        "mode": mode,
        "timestamp": Utc::now()
    })))
}


pub async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.memory.lock().await;
    let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
    Ok(HttpResponse::Ok().json(data))
}

pub async fn get_redis_history(state: web::Data<AppState>) -> Result<HttpResponse> {
    info!(
        operation = "redis_history_request",
        "Fetching last 60 seconds of data from Redis"
    );

    let mut conn = state
        .redis
        .lock()
        .await
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| {
            error!(
                error = %e,
                operation = "redis_connection",
                "Failed to establish Redis connection for history"
            );
            ApiError::Redis(format!("Connection failed: {e}"))
        })?;

    let keys: Vec<String> = conn.keys("sensor:*").await.map_err(|e| {
        error!(
            error = %e,
            operation = "redis_keys",
            "Failed to retrieve keys from Redis"
        );
        ApiError::Redis(format!("Failed to retrieve keys: {e}"))
    })?;

    if keys.is_empty() {
        info!(operation = "redis_history_empty", "No data found in Redis");
        return Ok(HttpResponse::Ok().json(Vec::<EnhancedSensorData>::new()));
    }

    let mut results: Vec<EnhancedSensorData> = Vec::new();

    for key in keys {
        match conn.get::<_, Option<String>>(&key).await {
            Ok(Some(value)) => match serde_json::from_str::<EnhancedSensorData>(&value) {
                Ok(data) => results.push(data),
                Err(e) => warn!(
                    error = %e,
                    key = %key,
                    operation = "redis_deserialize",
                    "Failed to deserialize data from Redis"
                ),
            },
            Ok(None) => warn!(
                key = %key,
                operation = "redis_get",
                "Key returned no value"
            ),
            Err(e) => warn!(
                error = %e,
                key = %key,
                operation = "redis_get",
                "Failed to get value from Redis"
            ),
        }
    }

    results.sort_by(|a, b| b.data.timestamp.cmp(&a.data.timestamp));
    let last_60: Vec<_> = results.into_iter().take(60).collect();

    Ok(HttpResponse::Ok().json(last_60))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    start: String,
    end: String,
}

/// Parse timestamp that could be in either RFC3339 or `MySQL` datetime format
fn parse_flexible_timestamp(timestamp: &str) -> Result<DateTime<Utc>, String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
        return Ok(dt.to_utc());
    }

    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
    }

    if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z") {
        return Ok(dt.to_utc());
    }

    Err(format!("Unable to parse timestamp: {timestamp}"))
}

/// Convert `MySQL` datetime format to RFC3339 for consistency
fn format_timestamp_rfc3339(timestamp: &str) -> String {
    match parse_flexible_timestamp(timestamp) {
        Ok(dt) => dt.to_rfc3339(),
        Err(_) => timestamp.to_string(),
    }
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::type_complexity)]
pub async fn get_history(
    state: web::Data<AppState>,
    query: web::Query<HistoryQuery>,
) -> Result<HttpResponse> {
    let start_time = parse_flexible_timestamp(&query.start)
        .map_err(|e| ApiError::Validation(format!("Invalid start timestamp: {e}")))?;

    let end_time = parse_flexible_timestamp(&query.end)
        .map_err(|e| ApiError::Validation(format!("Invalid end timestamp: {e}")))?;

    if start_time >= end_time {
        return Err(ApiError::Validation("Start time must be before end time".to_string()).into());
    }

    let start_mysql = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
    let end_mysql = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

    let mut conn = state
        .mysql
        .get_conn()
        .await
        .map_err(|e| ApiError::Database(format!("Connection failed: {e}")))?;

    let results: Vec<(f64, f64, f64, f64, bool, f64, String, String)> = conn
        .exec(
            r"SELECT temperature, humidity, noise, heart_rate, motion, stress_index, stress_level,
              CAST(timestamp AS CHAR) as timestamp
              FROM sensor_data
              WHERE timestamp >= ? AND timestamp <= ?
              ORDER BY timestamp DESC",
            (start_mysql, end_mysql),
        )
        .await
        .map_err(|e| ApiError::Database(format!("Query failed: {e}")))?;

    let data = results
        .into_iter()
        .map(
            |(
                temperature,
                humidity,
                noise,
                heart_rate,
                motion,
                stress_index,
                stress_level,
                timestamp,
            )| EnhancedSensorData {
                data: crate::models::SensorData {
                    temperature,
                    humidity,
                    noise,
                    heart_rate,
                    motion,
                    timestamp: format_timestamp_rfc3339(&timestamp),
                },
                stress_index,
                stress_level,
            },
        )
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(data))
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::manual_let_else)]
#[allow(clippy::single_match_else)]
#[allow(clippy::match_same_arms)]
pub async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.memory.lock().await;
    let latest = mem.back().cloned();
    drop(mem);

    let data = match latest {
        Some(d) => d,
        None => {
            return Ok(HttpResponse::NotFound().json(serde_json::json!({
                "error": "no_data",
                "message": "No sensor data available"
            })));
        }
    };

    let fhir_response = serde_json::json!({
        "resourceType": "Observation",
        "id": format!("stress-{}", data.data.timestamp),
        "status": "final",
        "effectiveDateTime": data.data.timestamp,
        "issued": Utc::now().to_rfc3339(),
        "valueQuantity": {
            "value": data.stress_index
        },
        "interpretation": [{
            "coding": [{
                "code": match data.stress_level.as_str() {
                    "Low" => "L",
                    "Moderate" => "N",
                    "High" => "H",
                    _ => "N"
                }
            }]
        }]
    });

    Ok(HttpResponse::Ok().json(fhir_response))
}

