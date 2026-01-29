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

//     // Get Redis connection
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

//     // Get all keys matching the sensor pattern
//     let keys: Vec<String> = conn
//         .keys("sensor:*")
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "redis_keys",
//                 "Failed to retrieve keys from Redis"
//             );
//             ApiError::Redis(format!("Failed to retrieve keys: {e}"))
//         })?;

//     if keys.is_empty() {
//         info!(
//             operation = "redis_history_empty",
//             "No data found in Redis"
//         );
//         return Ok(HttpResponse::Ok().json(Vec::<EnhancedSensorData>::new()));
//     }

//     // Get all values
//     let mut results: Vec<EnhancedSensorData> = Vec::new();

//     for key in keys {
//         match conn.get::<_, Option<String>>(&key).await {
//             Ok(Some(value)) => {
//                 match serde_json::from_str::<EnhancedSensorData>(&value) {
//                     Ok(data) => results.push(data),
//                     Err(e) => {
//                         warn!(
//                             error = %e,
//                             key = %key,
//                             operation = "redis_deserialize",
//                             "Failed to deserialize data from Redis"
//                         );
//                     }
//                 }
//             }
//             Ok(None) => {
//                 warn!(
//                     key = %key,
//                     operation = "redis_get",
//                     "Key returned no value"
//                 );
//             }
//             Err(e) => {
//                 warn!(
//                     error = %e,
//                     key = %key,
//                     operation = "redis_get",
//                     "Failed to get value from Redis"
//                 );
//             }
//         }
//     }

//     // Sort by timestamp (newest first) and take last 60 seconds
//     results.sort_by(|a, b| b.data.timestamp.cmp(&a.data.timestamp));
//     let last_60: Vec<_> = results.into_iter().take(60).collect();

//     info!(
//         operation = "redis_history_success",
//         count = %last_60.len(),
//         "Successfully retrieved data from Redis"
//     );

//     Ok(HttpResponse::Ok().json(last_60))
// }

// #[derive(Deserialize)]
// pub struct HistoryQuery {
//     start: String,
//     end: String,
// }

// /// Parse timestamp that could be in either RFC3339 or MySQL datetime format
// fn parse_flexible_timestamp(timestamp: &str) -> Result<DateTime<Utc>, String> {
//     // Try RFC3339 first (e.g., "2026-01-29T09:57:50Z")
//     if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
//         return Ok(dt.to_utc());
//     }

//     // Try MySQL datetime format (e.g., "2026-01-29 09:57:50")
//     if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
//         return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
//     }

//     // Try with timezone offset (e.g., "2026-01-29T09:57:50+00:00")
//     if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z") {
//         return Ok(dt.to_utc());
//     }

//     Err(format!("Unable to parse timestamp: {}", timestamp))
// }

// /// Convert MySQL datetime format to RFC3339 for consistency
// fn format_timestamp_rfc3339(timestamp: &str) -> String {
//     match parse_flexible_timestamp(timestamp) {
//         Ok(dt) => dt.to_rfc3339(),
//         Err(_) => timestamp.to_string(), // Return as-is if parsing fails
//     }
// }

// pub async fn get_history(
//     state: web::Data<AppState>,
//     query: web::Query<HistoryQuery>,
// ) -> Result<HttpResponse> {
//     info!(
//         operation = "mysql_history_request",
//         start = %query.start,
//         end = %query.end,
//         "Fetching historical data from MySQL"
//     );

//     // Parse and validate timestamps with flexible parsing
//     let start_time = parse_flexible_timestamp(&query.start)
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 timestamp = %query.start,
//                 operation = "timestamp_parse",
//                 "Failed to parse start timestamp"
//             );
//             ApiError::Validation(format!("Invalid start timestamp: {}", e))
//         })?;

//     let end_time = parse_flexible_timestamp(&query.end)
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 timestamp = %query.end,
//                 operation = "timestamp_parse",
//                 "Failed to parse end timestamp"
//             );
//             ApiError::Validation(format!("Invalid end timestamp: {}", e))
//         })?;

//     if start_time >= end_time {
//         return Err(ApiError::Validation(
//             "Start time must be before end time".to_string(),
//         )
//         .into());
//     }

//     // Convert to MySQL format for query (MySQL stores as "YYYY-MM-DD HH:MM:SS")
//     let start_mysql = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
//     let end_mysql = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

//     info!(
//         operation = "mysql_query_params",
//         start_mysql = %start_mysql,
//         end_mysql = %end_mysql,
//         "Converted timestamps to MySQL format"
//     );

//     // Get MySQL connection
//     let mut conn = state.mysql.get_conn().await.map_err(|e| {
//         error!(
//             error = %e,
//             operation = "mysql_connection",
//             "Failed to get MySQL connection from pool"
//         );
//         ApiError::Database(format!("Connection failed: {e}"))
//     })?;

//     // Query database - MySQL uses >= and <= for inclusive range
//     let results: Vec<(f64, f64, f64, f64, bool, f64, String, String)> = conn
//         .exec(
//             r"SELECT temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp
//               FROM sensor_data
//               WHERE timestamp >= ? AND timestamp <= ?
//               ORDER BY timestamp DESC",
//             (start_mysql, end_mysql),
//         )
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "mysql_query",
//                 start = %query.start,
//                 end = %query.end,
//                 "Failed to query sensor data from MySQL"
//             );
//             ApiError::Database(format!("Query failed: {e}"))
//         })?;

//     let data: Vec<EnhancedSensorData> = results
//         .into_iter()
//         .map(
//             |(temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)| {
//                 EnhancedSensorData {
//                     data: crate::models::SensorData {
//                         temperature,
//                         humidity,
//                         noise,
//                         heart_rate,
//                         motion,
//                         timestamp: format_timestamp_rfc3339(&timestamp), // Convert to RFC3339
//                     },
//                     stress_index,
//                     stress_level,
//                 }
//             },
//         )
//         .collect();

//     info!(
//         operation = "mysql_history_success",
//         count = %data.len(),
//         start = %query.start,
//         end = %query.end,
//         "Successfully retrieved historical data from MySQL"
//     );

//     Ok(HttpResponse::Ok().json(data))
// }

// pub async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
//     info!(
//         operation = "fhir_observation_request",
//         "Fetching latest observation in FHIR format"
//     );

//     // Get the latest data from memory
//     let mem = state.memory.lock().await;
//     let latest = mem.back().cloned();
//     drop(mem);

//     let data = match latest {
//         Some(d) => d,
//         None => {
//             warn!(
//                 operation = "fhir_observation_no_data",
//                 "No data available in memory for FHIR observation"
//             );
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
//         "category": [{
//             "coding": [{
//                 "system": "http://terminology.hl7.org/CodeSystem/observation-category",
//                 "code": "vital-signs",
//                 "display": "Vital Signs"
//             }]
//         }],
//         "code": {
//             "coding": [{
//                 "system": "http://loinc.org",
//                 "code": "85354-9",
//                 "display": "Stress Index"
//             }],
//             "text": "Stress Index"
//         },
//         "effectiveDateTime": data.data.timestamp,
//         "issued": Utc::now().to_rfc3339(),
//         "valueQuantity": {
//             "value": data.stress_index,
//             "unit": "index",
//             "system": "http://unitsofmeasure.org",
//             "code": "1"
//         },
//         "interpretation": [{
//             "coding": [{
//                 "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
//                 "code": match data.stress_level.as_str() {
//                     "Low" => "L",
//                     "Moderate" => "N",
//                     "High" => "H",
//                     _ => "N"
//                 },
//                 "display": data.stress_level.clone()
//             }],
//             "text": data.stress_level.clone()
//         }],
//         "component": [
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8310-5",
//                         "display": "Body temperature"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.temperature,
//                     "unit": "Cel",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "Cel"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8331-1",
//                         "display": "Relative humidity"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.humidity,
//                     "unit": "%",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "%"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8867-4",
//                         "display": "Heart rate"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.heart_rate,
//                     "unit": "beats/minute",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "/min"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "46254-2",
//                         "display": "Environmental noise"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.noise,
//                     "unit": "dB",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "dB"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "55421-8",
//                         "display": "Motion detected"
//                     }]
//                 },
//                 "valueBoolean": data.data.motion
//             }
//         ]
//     });

//     info!(
//         operation = "fhir_observation_success",
//         timestamp = %data.data.timestamp,
//         stress_level = %data.stress_level,
//         "Successfully generated FHIR observation"
//     );

//     Ok(HttpResponse::Ok().json(fhir_response))
// }

// version that has new api initial

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

//     // Get Redis connection
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

//     // Get all keys matching the sensor pattern
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

//     // Get all values
//     let mut results: Vec<EnhancedSensorData> = Vec::new();

//     for key in keys {
//         match conn.get::<_, Option<String>>(&key).await {
//             Ok(Some(value)) => match serde_json::from_str::<EnhancedSensorData>(&value) {
//                 Ok(data) => results.push(data),
//                 Err(e) => {
//                     warn!(
//                         error = %e,
//                         key = %key,
//                         operation = "redis_deserialize",
//                         "Failed to deserialize data from Redis"
//                     );
//                 }
//             },
//             Ok(None) => {
//                 warn!(
//                     key = %key,
//                     operation = "redis_get",
//                     "Key returned no value"
//                 );
//             }
//             Err(e) => {
//                 warn!(
//                     error = %e,
//                     key = %key,
//                     operation = "redis_get",
//                     "Failed to get value from Redis"
//                 );
//             }
//         }
//     }

//     // Sort by timestamp (newest first) and take last 60 seconds
//     results.sort_by(|a, b| b.data.timestamp.cmp(&a.data.timestamp));
//     let last_60: Vec<_> = results.into_iter().take(60).collect();

//     info!(
//         operation = "redis_history_success",
//         count = %last_60.len(),
//         "Successfully retrieved data from Redis"
//     );

//     Ok(HttpResponse::Ok().json(last_60))
// }

// #[derive(Deserialize)]
// pub struct HistoryQuery {
//     start: String,
//     end: String,
// }

// /// Parse timestamp that could be in either RFC3339 or MySQL datetime format
// fn parse_flexible_timestamp(timestamp: &str) -> Result<DateTime<Utc>, String> {
//     // Try RFC3339 first (e.g., "2026-01-29T09:57:50Z")
//     if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp) {
//         return Ok(dt.to_utc());
//     }

//     // Try MySQL datetime format (e.g., "2026-01-29 09:57:50")
//     if let Ok(naive_dt) = NaiveDateTime::parse_from_str(timestamp, "%Y-%m-%d %H:%M:%S") {
//         return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc));
//     }

//     // Try with timezone offset (e.g., "2026-01-29T09:57:50+00:00")
//     if let Ok(dt) = DateTime::parse_from_str(timestamp, "%Y-%m-%dT%H:%M:%S%z") {
//         return Ok(dt.to_utc());
//     }

//     Err(format!("Unable to parse timestamp: {}", timestamp))
// }

// /// Convert MySQL datetime format to RFC3339 for consistency
// fn format_timestamp_rfc3339(timestamp: &str) -> String {
//     match parse_flexible_timestamp(timestamp) {
//         Ok(dt) => dt.to_rfc3339(),
//         Err(_) => timestamp.to_string(), // Return as-is if parsing fails
//     }
// }

// pub async fn get_history(
//     state: web::Data<AppState>,
//     query: web::Query<HistoryQuery>,
// ) -> Result<HttpResponse> {
//     info!(
//         operation = "mysql_history_request",
//         start = %query.start,
//         end = %query.end,
//         "Fetching historical data from MySQL"
//     );

//     // Parse and validate timestamps with flexible parsing
//     let start_time = parse_flexible_timestamp(&query.start).map_err(|e| {
//         error!(
//             error = %e,
//             timestamp = %query.start,
//             operation = "timestamp_parse",
//             "Failed to parse start timestamp"
//         );
//         ApiError::Validation(format!("Invalid start timestamp: {}", e))
//     })?;

//     let end_time = parse_flexible_timestamp(&query.end).map_err(|e| {
//         error!(
//             error = %e,
//             timestamp = %query.end,
//             operation = "timestamp_parse",
//             "Failed to parse end timestamp"
//         );
//         ApiError::Validation(format!("Invalid end timestamp: {}", e))
//     })?;

//     if start_time >= end_time {
//         return Err(ApiError::Validation("Start time must be before end time".to_string()).into());
//     }

//     // Convert to MySQL format for query (MySQL stores as "YYYY-MM-DD HH:MM:SS")
//     let start_mysql = start_time.format("%Y-%m-%d %H:%M:%S").to_string();
//     let end_mysql = end_time.format("%Y-%m-%d %H:%M:%S").to_string();

//     info!(
//         operation = "mysql_query_params",
//         start_mysql = %start_mysql,
//         end_mysql = %end_mysql,
//         "Converted timestamps to MySQL format"
//     );

//     // Get MySQL connection
//     let mut conn = state.mysql.get_conn().await.map_err(|e| {
//         error!(
//             error = %e,
//             operation = "mysql_connection",
//             "Failed to get MySQL connection from pool"
//         );
//         ApiError::Database(format!("Connection failed: {e}"))
//     })?;

//     // First, let's check what type the timestamp column is by querying first
//     // This query works for both VARCHAR and DATETIME columns
//     // MySQL will auto-cast the string comparison if column is DATETIME
//     let results: Vec<(f64, f64, f64, f64, bool, f64, String, String)> = conn
//         .exec(
//             r"SELECT temperature, humidity, noise, heart_rate, motion, stress_index, stress_level,
//                      CAST(timestamp AS CHAR) as timestamp
//               FROM sensor_data
//               WHERE timestamp >= ? AND timestamp <= ?
//               ORDER BY timestamp DESC",
//             (start_mysql.clone(), end_mysql.clone()),
//         )
//         .await
//         .map_err(|e| {
//             error!(
//                 error = %e,
//                 operation = "mysql_query",
//                 start = %query.start,
//                 end = %query.end,
//                 start_mysql = %start_mysql,
//                 end_mysql = %end_mysql,
//                 "Failed to query sensor data from MySQL"
//             );
//             ApiError::Database(format!("Query failed: {e}"))
//         })?;

//     info!(
//         operation = "mysql_query_result",
//         row_count = %results.len(),
//         "MySQL query returned {} rows",
//         results.len()
//     );

//     let data: Vec<EnhancedSensorData> = results
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
//             )| {
//                 EnhancedSensorData {
//                     data: crate::models::SensorData {
//                         temperature,
//                         humidity,
//                         noise,
//                         heart_rate,
//                         motion,
//                         timestamp: format_timestamp_rfc3339(&timestamp), // Convert to RFC3339
//                     },
//                     stress_index,
//                     stress_level,
//                 }
//             },
//         )
//         .collect();

//     info!(
//         operation = "mysql_history_success",
//         count = %data.len(),
//         start = %query.start,
//         end = %query.end,
//         "Successfully retrieved historical data from MySQL"
//     );

//     Ok(HttpResponse::Ok().json(data))
// }

// pub async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
//     info!(
//         operation = "fhir_observation_request",
//         "Fetching latest observation in FHIR format"
//     );

//     // Get the latest data from memory
//     let mem = state.memory.lock().await;
//     let latest = mem.back().cloned();
//     drop(mem);

//     let data = match latest {
//         Some(d) => d,
//         None => {
//             warn!(
//                 operation = "fhir_observation_no_data",
//                 "No data available in memory for FHIR observation"
//             );
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
//         "category": [{
//             "coding": [{
//                 "system": "http://terminology.hl7.org/CodeSystem/observation-category",
//                 "code": "vital-signs",
//                 "display": "Vital Signs"
//             }]
//         }],
//         "code": {
//             "coding": [{
//                 "system": "http://loinc.org",
//                 "code": "85354-9",
//                 "display": "Stress Index"
//             }],
//             "text": "Stress Index"
//         },
//         "effectiveDateTime": data.data.timestamp,
//         "issued": Utc::now().to_rfc3339(),
//         "valueQuantity": {
//             "value": data.stress_index,
//             "unit": "index",
//             "system": "http://unitsofmeasure.org",
//             "code": "1"
//         },
//         "interpretation": [{
//             "coding": [{
//                 "system": "http://terminology.hl7.org/CodeSystem/v3-ObservationInterpretation",
//                 "code": match data.stress_level.as_str() {
//                     "Low" => "L",
//                     "Moderate" => "N",
//                     "High" => "H",
//                     _ => "N"
//                 },
//                 "display": data.stress_level.clone()
//             }],
//             "text": data.stress_level.clone()
//         }],
//         "component": [
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8310-5",
//                         "display": "Body temperature"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.temperature,
//                     "unit": "Cel",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "Cel"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8331-1",
//                         "display": "Relative humidity"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.humidity,
//                     "unit": "%",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "%"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "8867-4",
//                         "display": "Heart rate"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.heart_rate,
//                     "unit": "beats/minute",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "/min"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "46254-2",
//                         "display": "Environmental noise"
//                     }]
//                 },
//                 "valueQuantity": {
//                     "value": data.data.noise,
//                     "unit": "dB",
//                     "system": "http://unitsofmeasure.org",
//                     "code": "dB"
//                 }
//             },
//             {
//                 "code": {
//                     "coding": [{
//                         "system": "http://loinc.org",
//                         "code": "55421-8",
//                         "display": "Motion detected"
//                     }]
//                 },
//                 "valueBoolean": data.data.motion
//             }
//         ]
//     });

//     info!(
//         operation = "fhir_observation_success",
//         timestamp = %data.data.timestamp,
//         stress_level = %data.stress_level,
//         "Successfully generated FHIR observation"
//     );

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

pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
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
