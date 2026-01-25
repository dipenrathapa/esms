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

// use actix_cors::Cors;
// use actix_web::{web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv; // ✅ load .env
// use mysql_async::Opts;
// use rand::Rng;
// use serde::{Deserialize, Serialize};
// use std::collections::VecDeque;
// use std::env;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tokio::time::{interval, Duration};

// use mysql_async::{prelude::*, Pool};
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
//     let mut conn = client
//         .lock()
//         .await
//         .get_multiplexed_async_connection()
//         .await?;
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
//     let _: () = conn
//         .exec_drop(
//             query,
//             (
//                 data.data.temperature,
//                 data.data.humidity,
//                 data.data.noise,
//                 data.data.heart_rate,
//                 data.data.motion,
//                 data.stress_index,
//                 data.stress_level.clone(),
//                 data.data.timestamp.clone(),
//             ),
//         )
//         .await
//         .unwrap();
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

//     // Load environment variables from .env
//     dotenv().ok();

//     // Redis client from .env
//     let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set in .env");
//     let redis_client = redis::Client::open(redis_url).expect("Failed to connect to Redis");

//     // MySQL pool from .env
//     let mysql_url = env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL must be set in .env");
//     let mysql_pool = Pool::new(Opts::from_url(&mysql_url).unwrap());

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

// ---------------------- Imports ----------------------
// use actix_cors::Cors;
// use actix_web::{web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::{Opts, Pool, prelude::*};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::collections::VecDeque;
// use std::env;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use tokio::time::{interval, Duration};
// use tracing::{info, warn, error};
// use tracing_subscriber;

// // ---------------------- Models ----------------------
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

// impl EnhancedSensorData {
//     // Optional FHIR serialization (for future compliance)
//     fn to_fhir_observation(&self) -> serde_json::Value {
//         serde_json::json!({
//             "resourceType": "Observation",
//             "status": "final",
//             "effectiveDateTime": self.data.timestamp,
//             "valueQuantity": {
//                 "value": self.stress_index,
//                 "unit": "unitless"
//             },
//             "component": [
//                 {"code": "heart_rate", "value": self.data.heart_rate},
//                 {"code": "temperature", "value": self.data.temperature},
//                 {"code": "humidity", "value": self.data.humidity},
//                 {"code": "noise", "value": self.data.noise}
//             ]
//         })
//     }
// }

// // ---------------------- App State ----------------------
// struct AppState {
//     redis_client: Arc<Mutex<redis::Client>>,
//     mysql_pool: Pool,
//     in_memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>, // last 600 entries
// }

// // ---------------------- Stress Calculation ----------------------
// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let normalized_hr = (data.heart_rate - 60.0) / 100.0;
//     let temp_factor = data.temperature / 50.0;
//     let humidity_factor = data.humidity / 100.0;
//     let noise_factor = data.noise / 100.0;
//     (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
// }

// fn get_stress_level(stress_index: f64) -> String {
//     match stress_index {
//         x if x < 0.3 => "Low".to_string(),
//         x if x < 0.6 => "Moderate".to_string(),
//         _ => "High".to_string(),
//     }
// }

// // ---------------------- Sensor Simulation ----------------------
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

// // ---------------------- Redis Storage ----------------------
// async fn store_in_redis(
//     client: Arc<Mutex<redis::Client>>,
//     data: &EnhancedSensorData,
// ) -> redis::RedisResult<()> {
//     let mut conn = client
//         .lock()
//         .await
//         .get_multiplexed_async_connection()
//         .await?;
//     let key = format!("sensor:{}", data.data.timestamp);
//     conn.set_ex::<_, _, ()>(key, serde_json::to_string(data).unwrap(), 600)
//         .await?;
//     Ok(())
// }

// // ---------------------- MySQL Storage ----------------------
// async fn store_in_mysql(pool: &Pool, data: &EnhancedSensorData) {
//     match pool.get_conn().await {
//         Ok(mut conn) => {
//             let query = r"INSERT INTO sensor_data
//                 (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
//             if let Err(e) = conn.exec_drop(
//                 query,
//                 (
//                     data.data.temperature,
//                     data.data.humidity,
//                     data.data.noise,
//                     data.data.heart_rate,
//                     data.data.motion,
//                     data.stress_index,
//                     data.stress_level.clone(),
//                     data.data.timestamp.clone(),
//                 ),
//             ).await {
//                 error!("MySQL insert failed: {:?}", e);
//             }
//         }
//         Err(e) => error!("MySQL connection failed: {:?}", e),
//     }
// }

// // ---------------------- Sensor Task (Background) ----------------------
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

//         // Redis with retry
//         let redis_client = state.redis_client.clone();
//         let enhanced_clone = enhanced.clone();
//         tokio::spawn(async move {
//             for _ in 0..3 {
//                 if store_in_redis(redis_client.clone(), &enhanced_clone).await.is_ok() {
//                     break;
//                 }
//                 tokio::time::sleep(Duration::from_millis(100)).await;
//             }
//         });

//         // MySQL async
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
//     // Initialize logging
//     tracing_subscriber::fmt::init();
//     info!("🚀 Starting ESMS Backend...");

//     // Load .env
//     dotenv().ok();

//     // Redis
//     let redis_url = env::var("REDIS_URL")
//         .expect("REDIS_URL must be set in .env or Docker secrets");
//     let redis_client = redis::Client::open(redis_url).expect("Failed to connect to Redis");

//     // MySQL
//     let mysql_url = env::var("MYSQL_DATABASE_URL")
//         .expect("MYSQL_DATABASE_URL must be set in .env or Docker secrets");
//     let mysql_pool = Pool::new(Opts::from_url(&mysql_url).unwrap());

//     // App state
//     let state = web::Data::new(AppState {
//         redis_client: Arc::new(Mutex::new(redis_client)),
//         mysql_pool,
//         in_memory: Arc::new(Mutex::new(VecDeque::new())),
//     });

//     // Background sensor ingestion
//     let state_clone = state.clone();
//     tokio::spawn(async move {
//         sensor_task(state_clone).await;
//     });

//     // Start HTTP server
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

// use actix_cors::Cors;
// use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
// use chrono::Utc;
// use dotenv::dotenv;
// use mysql_async::prelude::Queryable;
// use mysql_async::{Opts, Pool};
// use rand::Rng;
// use redis::AsyncCommands;
// use serde::{Deserialize, Serialize};
// use std::{collections::VecDeque, env, sync::Arc};
// use tokio::{
//     sync::Mutex,
//     time::{interval, Duration},
// };
// use tracing::{info, warn};
// use tracing_subscriber::{fmt, EnvFilter};
// use validator::Validate;

// // ===================== Configuration =====================

// #[derive(Clone)]
// struct AppConfig {
//     redis_url: String,
//     mysql_url: String,
//     bind_addr: String,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//         }
//     }
// }

// // ===================== Models =====================

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

// // ===================== App State =====================

// struct AppState {
//     redis: Arc<Mutex<redis::Client>>,
//     mysql: Pool,
//     memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
// }

// // ===================== Business Logic =====================

// fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;

//     score.clamp(0.0, 1.0)
// }

// fn get_stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ===================== Sensor Simulation =====================

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

// // ===================== Background Task =====================

// async fn sensor_task(state: web::Data<AppState>) {
//     let mut tick = interval(Duration::from_secs(1));

//     loop {
//         tick.tick().await;

//         let data = simulate_sensor_data();
//         if let Err(e) = data.validate() {
//             warn!("Validation failed: {:?}", e);
//             continue;
//         }

//         let stress_index = calculate_stress_index(&data);
//         let enhanced = EnhancedSensorData {
//             stress_level: get_stress_level(stress_index),
//             stress_index,
//             data,
//         };

//         // In-memory cache
//         {
//             let mut mem = state.memory.lock().await;
//             mem.push_back(enhanced.clone());
//             if mem.len() > 600 {
//                 mem.pop_front();
//             }
//         }

//         // Redis (best-effort, NO ?)
//         let redis = state.redis.clone();
//         let payload = enhanced.clone();
//         tokio::spawn(async move {
//             if let Ok(mut conn) = redis
//                 .lock()
//                 .await
//                 .get_multiplexed_async_connection()
//                 .await
//             {
//                 if let Err(e) = conn
//                     .set_ex::<_, _, ()>(
//                         format!("sensor:{}", payload.data.timestamp),
//                         serde_json::to_string(&payload).unwrap(),
//                         600,
//                     )
//                     .await
//                 {
//                     warn!("Redis write failed: {:?}", e);
//                 }
//             }
//         });

//         // MySQL (best-effort)
//         let pool = state.mysql.clone();
//         let payload2 = enhanced.clone();
//         tokio::spawn(async move {
//             if let Ok(mut conn) = pool.get_conn().await {
//                 let _ = conn
//                     .exec_drop(
//                         r"INSERT INTO sensor_data
//                         (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
//                         (
//                             payload2.data.temperature,
//                             payload2.data.humidity,
//                             payload2.data.noise,
//                             payload2.data.heart_rate,
//                             payload2.data.motion,
//                             payload2.stress_index,
//                             payload2.stress_level,
//                             payload2.data.timestamp,
//                         ),
//                     )
//                     .await;
//             }
//         });
//     }
// }

// // ===================== API =====================

// async fn health() -> Result<HttpResponse> {
//     Ok(HttpResponse::Ok().json(serde_json::json!({
//         "status": "healthy",
//         "timestamp": Utc::now()
//     })))
// }

// async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
//     let mem = state.memory.lock().await;
//     let latest: Vec<_> = mem.iter().rev().take(60).cloned().collect();
//     Ok(HttpResponse::Ok().json(latest))
// }

// // ===================== Main =====================

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     dotenv().ok();

//     fmt()
//         .with_env_filter(EnvFilter::from_default_env())
//         .json()
//         .init();

//     let config = AppConfig::from_env();
//     info!("Starting ESMS backend");

//     let redis = redis::Client::open(config.redis_url).expect("Redis init failed");
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
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
//     sync::Mutex,
//     time::{interval, Duration},
// };
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
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//         }
//     }
// }

// // ======================================================
// // Error Handling
// // ======================================================

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
// // Background Task
// // ======================================================

// async fn sensor_task(state: web::Data<AppState>) {
//     let mut ticker = interval(Duration::from_secs(1));

//     loop {
//         ticker.tick().await;

//         let data = simulate_sensor_data();

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

//         // Redis (best effort)
//         let redis = state.redis.clone();
//         let redis_payload = enhanced.clone();
//         tokio::spawn(async move {
//             match redis.lock().await.get_multiplexed_async_connection().await {
//                 Ok(mut conn) => {
//                     if let Err(e) = conn
//                         .set_ex::<_, _, ()>(
//                             format!("sensor:{}", redis_payload.data.timestamp),
//                             serde_json::to_string(&redis_payload).unwrap(),
//                             600,
//                         )
//                         .await
//                     {
//                         warn!("Redis write failed: {:?}", e);
//                     }
//                 }
//                 Err(e) => warn!("Redis connection failed: {:?}", e),
//             }
//         });

//         // MySQL (best effort)
//         let pool = state.mysql.clone();
//         let db_payload = enhanced.clone();
//         tokio::spawn(async move {
//             match pool.get_conn().await {
//                 Ok(mut conn) => {
//                     if let Err(e) = conn
//                         .exec_drop(
//                             r#"INSERT INTO sensor_data
//                             (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//                             (
//                                 db_payload.data.temperature,
//                                 db_payload.data.humidity,
//                                 db_payload.data.noise,
//                                 db_payload.data.heart_rate,
//                                 db_payload.data.motion,
//                                 db_payload.stress_index,
//                                 db_payload.stress_level,
//                                 db_payload.data.timestamp,
//                             ),
//                         )
//                         .await
//                     {
//                         warn!("MySQL insert failed: {:?}", e);
//                     }
//                 }
//                 Err(e) => warn!("MySQL connection failed: {:?}", e),
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
//     info!("Starting ESMS backend");

//     let redis = redis::Client::open(config.redis_url).expect("Redis init failed");
//     let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

//     let state = web::Data::new(AppState {
//         redis: Arc::new(Mutex::new(redis)),
//         mysql,
//         memory: Arc::new(Mutex::new(VecDeque::new())),
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
// use serialport::SerialPort;
// use std::{collections::VecDeque, env, io::BufRead, io::BufReader, sync::Arc, time::Duration};
// use tokio::{
//     sync::Mutex,
//     time::{interval},
// };
// use tracing::{warn, info};
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
//     serial_port: String,
//     use_serial: bool,
// }

// impl AppConfig {
//     fn from_env() -> Self {
//         let use_serial = env::var("USE_SERIAL")
//             .unwrap_or_else(|_| "true".to_string())
//             .parse::<bool>()
//             .unwrap_or(true);

//         Self {
//             redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
//             mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
//             bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
//             serial_port: env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/cu.usbmodem13401".to_string()),
//             use_serial,
//         }
//     }
// }

// // ======================================================
// // Error Handling
// // ======================================================

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
// // Serial Reading
// // ======================================================

// fn read_sensor_from_serial(port_name: &str) -> Option<SensorData> {
//     if let Ok(port) = serialport::new(port_name, 9600).timeout(Duration::from_millis(500)).open() {
//         let mut reader = BufReader::new(port);
//         let mut line = String::new();

//         if let Ok(bytes) = reader.read_line(&mut line) {
//             if bytes > 0 {
//                 if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
//                     return Some(sensor);
//                 } else {
//                     warn!("Failed to parse JSON from serial: {}", line.trim());
//                 }
//             }
//         }
//     } else {
//         warn!("Failed to open serial port: {}", port_name);
//     }

//     None
// }

// // ======================================================
// // Background Task
// // ======================================================

// async fn sensor_task(state: web::Data<AppState>) {
//     let mut ticker = interval(Duration::from_secs(1));

//     loop {
//         ticker.tick().await;

//         // Decide whether to read from serial or simulate
//         let data = if state.config.use_serial {
//             read_sensor_from_serial(&state.config.serial_port).unwrap_or_else(simulate_sensor_data)
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

//         // Redis (best effort)
//         let redis = state.redis.clone();
//         let redis_payload = enhanced.clone();
//         tokio::spawn(async move {
//             match redis.lock().await.get_multiplexed_async_connection().await {
//                 Ok(mut conn) => {
//                     if let Err(e) = conn
//                         .set_ex::<_, _, ()>(
//                             format!("sensor:{}", redis_payload.data.timestamp),
//                             serde_json::to_string(&redis_payload).unwrap(),
//                             600,
//                         )
//                         .await
//                     {
//                         warn!("Redis write failed: {:?}", e);
//                     }
//                 }
//                 Err(e) => warn!("Redis connection failed: {:?}", e),
//             }
//         });

//         // MySQL (best effort)
//         let pool = state.mysql.clone();
//         let db_payload = enhanced.clone();
//         tokio::spawn(async move {
//             match pool.get_conn().await {
//                 Ok(mut conn) => {
//                     if let Err(e) = conn
//                         .exec_drop(
//                             r#"INSERT INTO sensor_data
//                             (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
//                             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
//                             (
//                                 db_payload.data.temperature,
//                                 db_payload.data.humidity,
//                                 db_payload.data.noise,
//                                 db_payload.data.heart_rate,
//                                 db_payload.data.motion,
//                                 db_payload.stress_index,
//                                 db_payload.stress_level,
//                                 db_payload.data.timestamp,
//                             ),
//                         )
//                         .await
//                     {
//                         warn!("MySQL insert failed: {:?}", e);
//                     }
//                 }
//                 Err(e) => warn!("MySQL connection failed: {:?}", e),
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

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use chrono::Utc;
use dotenv::dotenv;
use mysql_async::{prelude::Queryable, Opts, Pool};
use rand::Rng;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, env, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
    sync::Mutex,
    time::{interval, Duration},
};
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};
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
}

// ======================================================
// Error Handling
// ======================================================

#[derive(thiserror::Error, Debug)]
enum ApiError {
    #[error("Internal server error")]
    Internal,
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "internal_error"
        }))
    }
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
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(_) => {
                    if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
                        Some(sensor)
                    } else {
                        warn!("Failed to parse JSON from TCP: {}", line.trim());
                        None
                    }
                }
                Err(e) => {
                    warn!("TCP read error: {:?}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("TCP connect failed: {:?}", e);
            None
        }
    }
}

// ======================================================
// Background Task
// ======================================================

async fn sensor_task(state: web::Data<AppState>) {
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        let data = if state.config.use_serial {
            read_sensor_from_tcp(&state.config.serial_tcp_host, state.config.serial_tcp_port)
                .await
                .unwrap_or_else(simulate_sensor_data)
        } else {
            simulate_sensor_data()
        };

        if let Err(e) = data.validate() {
            warn!("Validation failed: {:?}", e);
            continue;
        }

        let index = calculate_stress_index(&data);
        let enhanced = EnhancedSensorData {
            stress_index: index,
            stress_level: stress_level(index),
            data,
        };

        // In-memory fallback
        {
            let mut mem = state.memory.lock().await;
            mem.push_back(enhanced.clone());
            if mem.len() > 600 {
                mem.pop_front();
            }
        }

        // Redis
        let redis = state.redis.clone();
        let redis_payload = enhanced.clone();
        tokio::spawn(async move {
            if let Ok(mut conn) = redis.lock().await.get_multiplexed_async_connection().await {
                if let Err(e) = conn
                    .set_ex::<_, _, ()>(
                        format!("sensor:{}", redis_payload.data.timestamp),
                        serde_json::to_string(&redis_payload).unwrap(),
                        600,
                    )
                    .await
                {
                    warn!("Redis write failed: {:?}", e);
                }
            }
        });

        // MySQL
        let pool = state.mysql.clone();
        let db_payload = enhanced.clone();
        tokio::spawn(async move {
            if let Ok(mut conn) = pool.get_conn().await {
                let res = conn
                    .exec_drop(
                        r#"INSERT INTO sensor_data
                        (temperature, humidity, noise, heart_rate, motion, stress_index, stress_level, timestamp)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
                        (
                            db_payload.data.temperature,
                            db_payload.data.humidity,
                            db_payload.data.noise,
                            db_payload.data.heart_rate,
                            db_payload.data.motion,
                            db_payload.stress_index,
                            db_payload.stress_level,
                            db_payload.data.timestamp,
                        ),
                    )
                    .await;
                if let Err(e) = res {
                    warn!("MySQL insert failed: {:?}", e);
                }
            }
        });
    }
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

    let config = AppConfig::from_env();
    info!("Starting ESMS backend (use_serial={})", config.use_serial);

    let redis = redis::Client::open(config.redis_url.clone()).expect("Redis init failed");
    let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

    let state = web::Data::new(AppState {
        redis: Arc::new(Mutex::new(redis)),
        mysql,
        memory: Arc::new(Mutex::new(VecDeque::new())),
        config: config.clone(),
    });

    tokio::spawn(sensor_task(state.clone()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .app_data(state.clone())
            .route("/health", web::get().to(health))
            .route("/api/realtime", web::get().to(get_realtime))
    })
    .bind(&config.bind_addr)?
    .run()
    .await
}
