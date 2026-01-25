// use actix_web::{web, App, HttpServer, HttpResponse, Result};
// use actix_cors::Cors;
// use serde::{Deserialize, Serialize};
// use tokio::sync::Mutex;
// use std::sync::Arc;
// use chrono::{DateTime, Utc};
// use std::collections::VecDeque;
// use tokio::time::{interval, Duration};

// // Sensor data structure matching Arduino output
// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct SensorData {
//     temperature: f64,
//     humidity: f64,
//     noise: f64,
//     heart_rate: f64,
//     motion: bool,
//     timestamp: String,
// }

// // FHIR-compatible observation structure
// #[derive(Debug, Serialize)]
// struct Observation {
//     resourceType: String,
//     id: String,
//     status: String,
//     category: Vec<Category>,
//     code: Code,
//     effectiveDateTime: String,
//     valueQuantity: ValueQuantity,
//     component: Vec<Component>,
// }

// #[derive(Debug, Serialize)]
// struct Category {
//     coding: Vec<Coding>,
// }

// #[derive(Debug, Serialize)]
// struct Coding {
//     system: String,
//     code: String,
//     display: String,
// }

// #[derive(Debug, Serialize)]
// struct Code {
//     coding: Vec<Coding>,
//     text: String,
// }

// #[derive(Debug, Serialize)]
// struct ValueQuantity {
//     value: f64,
//     unit: String,
// }

// #[derive(Debug, Serialize)]
// struct Component {
//     code: Code,
//     valueQuantity: ValueQuantity,
// }

// // Enhanced response with stress index
// #[derive(Debug, Clone, Serialize)]
// struct EnhancedSensorData {
//     #[serde(flatten)]
//     data: SensorData,
//     stress_index: f64,
//     stress_level: String,
// }

// // Application state
// struct AppState {
//     redis_data: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
//     mysql_data: Arc<Mutex<Vec<EnhancedSensorData>>>,
// }

// // Calculate stress index
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let normalized_hr = (data.heart_rate - 60.0) / 100.0;
//     let temp_factor = data.temperature / 50.0;
//     let humidity_factor = data.humidity / 100.0;
//     let noise_factor = data.noise / 100.0;

//     (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
// }

// // Determine stress level
// fn get_stress_level(stress_index: f64) -> String {
//     if stress_index < 0.3 {
//         "Low".to_string()
//     } else if stress_index < 0.6 {
//         "Moderate".to_string()
//     } else {
//         "High".to_string()
//     }
// }

// // Convert to FHIR format
// fn to_fhir_observation(data: &EnhancedSensorData) -> Observation {
//     Observation {
//         resourceType: "Observation".to_string(),
//         id: format!("stress-{}", data.data.timestamp),
//         status: "final".to_string(),
//         category: vec![Category {
//             coding: vec![Coding {
//                 system: "http://terminology.hl7.org/CodeSystem/observation-category".to_string(),
//                 code: "vital-signs".to_string(),
//                 display: "Vital Signs".to_string(),
//             }],
//         }],
//         code: Code {
//             coding: vec![Coding {
//                 system: "http://loinc.org".to_string(),
//                 code: "85354-9".to_string(),
//                 display: "Stress Index".to_string(),
//             }],
//             text: "Environmental Stress Index".to_string(),
//         },
//         effectiveDateTime: data.data.timestamp.clone(),
//         valueQuantity: ValueQuantity {
//             value: data.stress_index,
//             unit: "index".to_string(),
//         },
//         component: vec![
//             Component {
//                 code: Code {
//                     coding: vec![Coding {
//                         system: "http://loinc.org".to_string(),
//                         code: "8310-5".to_string(),
//                         display: "Body temperature".to_string(),
//                     }],
//                     text: "Temperature".to_string(),
//                 },
//                 valueQuantity: ValueQuantity {
//                     value: data.data.temperature,
//                     unit: "Cel".to_string(),
//                 },
//             },
//             Component {
//                 code: Code {
//                     coding: vec![Coding {
//                         system: "http://loinc.org".to_string(),
//                         code: "8867-4".to_string(),
//                         display: "Heart rate".to_string(),
//                     }],
//                     text: "Heart Rate".to_string(),
//                 },
//                 valueQuantity: ValueQuantity {
//                     value: data.data.heart_rate,
//                     unit: "bpm".to_string(),
//                 },
//             },
//         ],
//     }
// }

// // Simulate sensor data (for cloud/Codespaces)
// fn simulate_sensor_data() -> SensorData {
//     use rand::Rng;
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

// // Read from Arduino serial port
// async fn read_serial_data(port_name: &str) -> Option<SensorData> {
//     use tokio_serial::SerialPortBuilderExt;

//     match tokio_serial::new(port_name, 9600).open_native_async() {
//         Ok(mut port) => {
//             use tokio::io::AsyncReadExt;
//             let mut buf = vec![0u8; 1024];

//             match port.read(&mut buf).await {
//                 Ok(n) if n > 0 => {
//                     let data_str = String::from_utf8_lossy(&buf[..n]);
//                     if let Ok(data) = serde_json::from_str::<SensorData>(&data_str) {
//                         return Some(data);
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         Err(_) => {}
//     }
//     None
// }

// // Background task to ingest sensor data
// async fn sensor_ingestion_task(state: web::Data<AppState>) {
//     let mut interval = interval(Duration::from_secs(1));
//     let serial_port = std::env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/cu.usbmodem113401".to_string());

//     loop {
//         interval.tick().await;

//         // Try to read from serial, fallback to simulation
//         let sensor_data = match read_serial_data(&serial_port).await {
//             Some(data) => data,
//             None => simulate_sensor_data(),
//         };

//         let stress_index = calculate_stress_index(&sensor_data);
//         let stress_level = get_stress_level(stress_index);

//         let enhanced_data = EnhancedSensorData {
//             data: sensor_data,
//             stress_index,
//             stress_level,
//         };

//         // Store in Redis (last 10 minutes = 600 entries)
//         {
//             let mut redis = state.redis_data.lock().await;
//             redis.push_back(enhanced_data.clone());
//             if redis.len() > 600 {
//                 redis.pop_front();
//             }
//         }

//         // Store in MySQL (historical)
//         {
//             let mut mysql = state.mysql_data.lock().await;
//             mysql.push(enhanced_data);
//         }
//     }
// }

// // API endpoint: real-time data
// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let redis = state.redis_data.lock().await;
//     let recent_data: Vec<EnhancedSensorData> = redis.iter().rev().take(60).cloned().collect();

//     Ok(HttpResponse::Ok().json(recent_data))
// }

// // API endpoint: historical data
// async fn get_history(
//     state: web::Data<AppState>,
//     query: web::Query<std::collections::HashMap<String, String>>,
// ) -> Result<HttpResponse> {
//     let mysql = state.mysql_data.lock().await;

//     // Simple time filtering (can be enhanced)
//     let filtered_data: Vec<EnhancedSensorData> = if let (Some(start), Some(end)) = (query.get("start"), query.get("end")) {
//         mysql.iter()
//             .filter(|d| d.data.timestamp >= *start && d.data.timestamp <= *end)
//             .cloned()
//             .collect()
//     } else {
//         mysql.clone()
//     };

//     Ok(HttpResponse::Ok().json(filtered_data))
// }

// // API endpoint: FHIR observation
// async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let redis = state.redis_data.lock().await;

//     if let Some(latest) = redis.back() {
//         let observation = to_fhir_observation(latest);
//         Ok(HttpResponse::Ok().json(observation))
//     } else {
//         Ok(HttpResponse::NotFound().json(serde_json::json!({
//             "error": "No data available"
//         })))
//     }
// }

// // Health check endpoint
// async fn health_check() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now().to_rfc3339()
//     })))
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("🚀 Starting ESMS Backend Server...");

//     let state = web::Data::new(AppState {
//         redis_data: Arc::new(Mutex::new(VecDeque::new())),
//         mysql_data: Arc::new(Mutex::new(Vec::new())),
//     });

//     // Start background sensor ingestion
//     let state_clone = state.clone();
//     tokio::spawn(async move {
//         sensor_ingestion_task(state_clone).await;
//     });

//     println!("✅ Backend listening on http://0.0.0.0:8080");

//     HttpServer::new(move || {
//         let cors = Cors::permissive();

//         App::new()
//             .wrap(cors)
//             .app_data(state.clone())
//             .route("/health", web::get().to(health_check))
//             .route("/api/realtime", web::get().to(get_realtime))
//             .route("/api/history", web::get().to(get_history))
//             .route("/api/fhir/observation", web::get().to(get_fhir_observation))
//     })
//     .bind("0.0.0.0:8080")?
//     .run()
//     .await
// }

// use actix_web::{web, App, HttpServer, HttpResponse, Result};
// use actix_cors::Cors;
// use serde::{Deserialize, Serialize};
// use chrono::Utc;
// use tokio::time::{interval, Duration};
// use rand::Rng;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use std::collections::VecDeque;

// use mysql_async::{Pool, prelude::*};
// use redis::AsyncCommands;

// // ---------------------- Data structures ----------------------
// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct SensorData {
//     temperature: f64,
//     humidity: f64,
//     noise: f64,
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

// // FHIR-compatible Observation
// #[derive(Debug, Serialize)]
// struct Observation {
//     resourceType: String,
//     id: String,
//     status: String,
//     category: Vec<Category>,
//     code: Code,
//     effectiveDateTime: String,
//     valueQuantity: ValueQuantity,
//     component: Vec<Component>,
// }

// #[derive(Debug, Serialize)]
// struct Category { coding: Vec<Coding> }
// #[derive(Debug, Serialize)]
// struct Coding { system: String, code: String, display: String }
// #[derive(Debug, Serialize)]
// struct Code { coding: Vec<Coding>, text: String }
// #[derive(Debug, Serialize)]
// struct ValueQuantity { value: f64, unit: String }
// #[derive(Debug, Serialize)]
// struct Component { code: Code, valueQuantity: ValueQuantity }

// // ---------------------- App State ----------------------
// struct AppState {
//     redis_client: Arc<Mutex<redis::Client>>,
//     mysql_pool: Pool,
//     in_memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>, // last 600 entries
// }

// // ---------------------- Stress calculation ----------------------
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let normalized_hr = (data.heart_rate - 60.0) / 100.0;
//     let temp_factor = data.temperature / 50.0;
//     let humidity_factor = data.humidity / 100.0;
//     let noise_factor = data.noise / 100.0;
//     (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
// }

// fn get_stress_level(stress_index: f64) -> String {
//     if stress_index < 0.3 { "Low".to_string() }
//     else if stress_index < 0.6 { "Moderate".to_string() }
//     else { "High".to_string() }
// }

// // ---------------------- Simulate sensor data ----------------------
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

// // ---------------------- Store in MySQL ----------------------
// async fn store_in_mysql(pool: &Pool, data: &EnhancedSensorData) {
//     let mut conn = pool.get_conn().await.unwrap();
//     let query = r"INSERT INTO sensor_data
//         (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//         VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
//     let _ : () = conn.exec_drop(
//         query,
//         (
//             data.data.temperature,
//             data.data.humidity,
//             data.data.noise,
//             data.data.heart_rate,
//             data.data.motion,
//             data.stress_index,
//             data.stress_level.clone(),
//             data.data.timestamp.clone(),
//         )
//     ).await.unwrap();
// }

// // ---------------------- Store in Redis ----------------------
// async fn store_in_redis(client: Arc<Mutex<redis::Client>>, data: &EnhancedSensorData) {
//     let mut conn = client.lock().await.get_async_connection().await.unwrap();
//     let key = format!("sensor:{}", data.data.timestamp);
//     let _: () = conn.set_ex(key, serde_json::to_string(data).unwrap(), 600).await.unwrap();
// }

// // ---------------------- Convert to FHIR ----------------------
// fn to_fhir_observation(data: &EnhancedSensorData) -> Observation {
//     Observation {
//         resourceType: "Observation".to_string(),
//         id: format!("stress-{}", data.data.timestamp),
//         status: "final".to_string(),
//         category: vec![Category {
//             coding: vec![Coding {
//                 system: "http://terminology.hl7.org/CodeSystem/observation-category".to_string(),
//                 code: "vital-signs".to_string(),
//                 display: "Vital Signs".to_string(),
//             }],
//         }],
//         code: Code {
//             coding: vec![Coding {
//                 system: "http://loinc.org".to_string(),
//                 code: "85354-9".to_string(),
//                 display: "Stress Index".to_string(),
//             }],
//             text: "Environmental Stress Index".to_string(),
//         },
//         effectiveDateTime: data.data.timestamp.clone(),
//         valueQuantity: ValueQuantity { value: data.stress_index, unit: "index".to_string() },
//         component: vec![
//             Component {
//                 code: Code {
//                     coding: vec![Coding {
//                         system: "http://loinc.org".to_string(),
//                         code: "8310-5".to_string(),
//                         display: "Body temperature".to_string(),
//                     }],
//                     text: "Temperature".to_string(),
//                 },
//                 valueQuantity: ValueQuantity { value: data.data.temperature, unit: "Cel".to_string() },
//             },
//             Component {
//                 code: Code {
//                     coding: vec![Coding {
//                         system: "http://loinc.org".to_string(),
//                         code: "8867-4".to_string(),
//                         display: "Heart rate".to_string(),
//                     }],
//                     text: "Heart Rate".to_string(),
//                 },
//                 valueQuantity: ValueQuantity { value: data.data.heart_rate, unit: "bpm".to_string() },
//             },
//         ],
//     }
// }

// // ---------------------- Background ingestion ----------------------
// async fn sensor_task(state: web::Data<AppState>) {
//     let mut interval = interval(Duration::from_secs(1));

//     loop {
//         interval.tick().await;

//         let sensor_data = simulate_sensor_data();
//         let stress_index = calculate_stress_index(&sensor_data);
//         let stress_level = get_stress_level(stress_index);

//         let enhanced = EnhancedSensorData {
//             data: sensor_data,
//             stress_index,
//             stress_level,
//         };

//         // In-memory cache
//         {
//             let mut mem = state.in_memory.lock().await;
//             mem.push_back(enhanced.clone());
//             if mem.len() > 600 { mem.pop_front(); }
//         }

//         // MySQL insert
//         let pool = state.mysql_pool.clone();
//         let enhanced_clone = enhanced.clone();
//         tokio::spawn(async move {
//             store_in_mysql(&pool, &enhanced_clone).await;
//         });

//         // Redis store
//         let redis_client = state.redis_client.clone();
//         let enhanced_clone2 = enhanced.clone();
//         tokio::spawn(async move {
//             store_in_redis(redis_client, &enhanced_clone2).await;
//         });
//     }
// }

// // ---------------------- API Endpoints ----------------------
// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.in_memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// async fn get_history(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mut conn = state.mysql_pool.get_conn().await.unwrap();
//     let rows = conn.query_map(
//         "SELECT temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp FROM sensor_data ORDER BY timestamp DESC LIMIT 1000",
//         |(temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)| {
//             EnhancedSensorData {
//                 data: SensorData {
//                     temperature,
//                     humidity,
//                     noise,
//                     heart_rate,
//                     motion,
//                     timestamp,
//                 },
//                 stress_index,
//                 stress_level,
//             }
//         }
//     ).await.unwrap();
//     Ok(HttpResponse::Ok().json(rows))
// }

// async fn get_fhir(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.in_memory.lock().await;
//     if let Some(latest) = mem.back() {
//         Ok(HttpResponse::Ok().json(to_fhir_observation(latest)))
//     } else {
//         Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "No data"})))
//     }
// }

// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({"status": "healthy"})))
// }

// // ---------------------- Main ----------------------
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("🚀 Starting ESMS Backend...");

//     let redis_client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
//     let mysql_pool = Pool::new("mysql://root:password@127.0.0.1:3306/esms_db");

//     let state = web::Data::new(AppState {
//         redis_client: Arc::new(Mutex::new(redis_client)),
//         mysql_pool,
//         in_memory: Arc::new(Mutex::new(VecDeque::new())),
//     });

//     // Start ingestion task
//     let state_clone = state.clone();
//     tokio::spawn(async move { sensor_task(state_clone).await });

//     HttpServer::new(move || {
//         let cors = Cors::permissive();
//         App::new()
//             .wrap(cors)
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//             .route("/api/history", web::get().to(get_history))
//             .route("/api/fhir/observation", web::get().to(get_fhir))
//     })
//     .bind("0.0.0.0:8080")?
//     .run()
//     .await
// }

// use actix_web::{web, App, HttpServer, HttpResponse, Result};
// use actix_cors::Cors;
// use serde::{Deserialize, Serialize};
// use chrono::Utc;
// use tokio::time::{interval, Duration};
// use rand::Rng;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use std::collections::VecDeque;

// use mysql_async::{Pool, prelude::*};
// use redis::AsyncCommands;

// // ---------------------- Data structures ----------------------
// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct SensorData {
//     temperature: f64,
//     humidity: f64,
//     noise: f64,
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

// // ---------------------- App State ----------------------
// struct AppState {
//     redis_client: Arc<Mutex<redis::Client>>,
//     mysql_pool: Pool,
//     in_memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>, // last 600 entries
// }

// // ---------------------- Stress calculation ----------------------
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let normalized_hr = (data.heart_rate - 60.0) / 100.0;
//     let temp_factor = data.temperature / 50.0;
//     let humidity_factor = data.humidity / 100.0;
//     let noise_factor = data.noise / 100.0;
//     (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
// }

// fn get_stress_level(stress_index: f64) -> String {
//     if stress_index < 0.3 {
//         "Low".to_string()
//     } else if stress_index < 0.6 {
//         "Moderate".to_string()
//     } else {
//         "High".to_string()
//     }
// }

// // ---------------------- Simulate sensor data ----------------------
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

// // ---------------------- Store in Redis ----------------------
// async fn store_in_redis(
//     client: Arc<Mutex<redis::Client>>,
//     data: &EnhancedSensorData,
// ) -> redis::RedisResult<()> {
//     // Use multiplexed async connection
//     let mut conn = client.lock().await.get_multiplexed_async_connection().await?;
//     let key = format!("sensor:{}", data.data.timestamp);
//     conn.set_ex::<_, _, ()>(key, serde_json::to_string(data).unwrap(), 600)
//         .await?;
//     Ok(())
// }

// // ---------------------- Store in MySQL ----------------------
// async fn store_in_mysql(pool: &Pool, data: &EnhancedSensorData) {
//     let mut conn = pool.get_conn().await.unwrap();
//     let query = r"INSERT INTO sensor_data
//         (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//         VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
//     let _ : () = conn.exec_drop(
//         query,
//         (
//             data.data.temperature,
//             data.data.humidity,
//             data.data.noise,
//             data.data.heart_rate,
//             data.data.motion,
//             data.stress_index,
//             data.stress_level.clone(),
//             data.data.timestamp.clone(),
//         )
//     ).await.unwrap();
// }

// // ---------------------- Background ingestion ----------------------
// async fn sensor_task(state: web::Data<AppState>) {
//     let mut interval = interval(Duration::from_secs(1));

//     loop {
//         interval.tick().await;

//         let sensor_data = simulate_sensor_data();
//         let stress_index = calculate_stress_index(&sensor_data);
//         let stress_level = get_stress_level(stress_index);

//         let enhanced = EnhancedSensorData {
//             data: sensor_data,
//             stress_index,
//             stress_level,
//         };

//         // In-memory cache
//         {
//             let mut mem = state.in_memory.lock().await;
//             mem.push_back(enhanced.clone());
//             if mem.len() > 600 {
//                 mem.pop_front();
//             }
//         }

//         // Redis
//         let redis_client = state.redis_client.clone();
//         let enhanced_clone = enhanced.clone();
//         tokio::spawn(async move {
//             if let Err(e) = store_in_redis(redis_client, &enhanced_clone).await {
//                 eprintln!("Redis error: {:?}", e);
//             }
//         });

//         // MySQL
//         let mysql_pool = state.mysql_pool.clone();
//         let enhanced_clone2 = enhanced.clone();
//         tokio::spawn(async move {
//             store_in_mysql(&mysql_pool, &enhanced_clone2).await;
//         });
//     }
// }

// // ---------------------- API Endpoints ----------------------
// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.in_memory.lock().await;
//     let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(data))
// }

// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({"status": "healthy"})))
// }

// // ---------------------- Main ----------------------
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("🚀 Starting ESMS Backend...");

//     // Redis client (Docker service name: redis)
//     let redis_client = redis::Client::open("redis://redis:6379").unwrap();

//     // MySQL pool (Docker service name: mysql)
//     let mysql_pool = Pool::new("mysql://esms_user:esms_pass@mysql:3306/esms_db");

//     let state = web::Data::new(AppState {
//         redis_client: Arc::new(Mutex::new(redis_client)),
//         mysql_pool,
//         in_memory: Arc::new(Mutex::new(VecDeque::new())),
//     });

//     // Start sensor ingestion task
//     let state_clone = state.clone();
//     tokio::spawn(async move {
//         sensor_task(state_clone).await;
//     });

//     HttpServer::new(move || {
//         let cors = Cors::permissive();
//         App::new()
//             .wrap(cors)
//             .app_data(state.clone())
//             .route("/health", web::get().to(health))
//             .route("/api/realtime", web::get().to(get_realtime))
//     })
//     .bind("0.0.0.0:8080")?
//     .run()
//     .await
// }

use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Result};
use chrono::Utc;
use dotenv::dotenv; // ✅ load .env
use mysql_async::Opts;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

use mysql_async::{prelude::*, Pool};
use redis::AsyncCommands;

// ---------------------- Data structures ----------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorData {
    temperature: f64,
    humidity: f64,
    noise: f64,
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

// ---------------------- App State ----------------------
struct AppState {
    redis_client: Arc<Mutex<redis::Client>>,
    mysql_pool: Pool,
    in_memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>, // last 600 entries
}

// ---------------------- Stress calculation ----------------------
fn calculate_stress_index(data: &SensorData) -> f64 {
    let normalized_hr = (data.heart_rate - 60.0) / 100.0;
    let temp_factor = data.temperature / 50.0;
    let humidity_factor = data.humidity / 100.0;
    let noise_factor = data.noise / 100.0;
    (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
}

fn get_stress_level(stress_index: f64) -> String {
    if stress_index < 0.3 {
        "Low".to_string()
    } else if stress_index < 0.6 {
        "Moderate".to_string()
    } else {
        "High".to_string()
    }
}

// ---------------------- Simulate sensor data ----------------------
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

// ---------------------- Store in Redis ----------------------
async fn store_in_redis(
    client: Arc<Mutex<redis::Client>>,
    data: &EnhancedSensorData,
) -> redis::RedisResult<()> {
    let mut conn = client
        .lock()
        .await
        .get_multiplexed_async_connection()
        .await?;
    let key = format!("sensor:{}", data.data.timestamp);
    conn.set_ex::<_, _, ()>(key, serde_json::to_string(data).unwrap(), 600)
        .await?;
    Ok(())
}

// ---------------------- Store in MySQL ----------------------
async fn store_in_mysql(pool: &Pool, data: &EnhancedSensorData) {
    let mut conn = pool.get_conn().await.unwrap();
    let query = r"INSERT INTO sensor_data
        (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
    let _: () = conn
        .exec_drop(
            query,
            (
                data.data.temperature,
                data.data.humidity,
                data.data.noise,
                data.data.heart_rate,
                data.data.motion,
                data.stress_index,
                data.stress_level.clone(),
                data.data.timestamp.clone(),
            ),
        )
        .await
        .unwrap();
}

// ---------------------- Background ingestion ----------------------
async fn sensor_task(state: web::Data<AppState>) {
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let sensor_data = simulate_sensor_data();
        let stress_index = calculate_stress_index(&sensor_data);
        let stress_level = get_stress_level(stress_index);

        let enhanced = EnhancedSensorData {
            data: sensor_data,
            stress_index,
            stress_level,
        };

        // In-memory cache
        {
            let mut mem = state.in_memory.lock().await;
            mem.push_back(enhanced.clone());
            if mem.len() > 600 {
                mem.pop_front();
            }
        }

        // Redis
        let redis_client = state.redis_client.clone();
        let enhanced_clone = enhanced.clone();
        tokio::spawn(async move {
            if let Err(e) = store_in_redis(redis_client, &enhanced_clone).await {
                eprintln!("Redis error: {:?}", e);
            }
        });

        // MySQL
        let mysql_pool = state.mysql_pool.clone();
        let enhanced_clone2 = enhanced.clone();
        tokio::spawn(async move {
            store_in_mysql(&mysql_pool, &enhanced_clone2).await;
        });
    }
}

// ---------------------- API Endpoints ----------------------
async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.in_memory.lock().await;
    let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
    Ok(HttpResponse::Ok().json(data))
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "healthy"})))
}

// ---------------------- Main ----------------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting ESMS Backend...");

    // Load environment variables from .env
    dotenv().ok();

    // Redis client from .env
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env");
    let redis_client = redis::Client::open(redis_url).expect("Failed to connect to Redis");

    // MySQL pool from .env
    let mysql_url = env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL must be set in .env");
    let mysql_pool = Pool::new(Opts::from_url(&mysql_url).unwrap());

    let state = web::Data::new(AppState {
        redis_client: Arc::new(Mutex::new(redis_client)),
        mysql_pool,
        in_memory: Arc::new(Mutex::new(VecDeque::new())),
    });

    // Start sensor ingestion task
    let state_clone = state.clone();
    tokio::spawn(async move {
        sensor_task(state_clone).await;
    });

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/health", web::get().to(health))
            .route("/api/realtime", web::get().to(get_realtime))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
