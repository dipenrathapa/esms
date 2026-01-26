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
// mod lib; // imports lib.rs (can also rename lib.rs to esms_backend.rs and use as crate)

// use crate::lib::{calculate_stress_index, stress_level, SensorData};

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



use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Result};
use chrono::Utc;
use dotenv::dotenv;
use mysql_async::{Opts, Pool};
use rand::Rng;
use redis::AsyncCommands;
use std::{collections::VecDeque, env, sync::Arc};
use tokio::{io::{AsyncBufReadExt, BufReader}, net::TcpStream, sync::Mutex, time::{interval, Duration}};
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

mod lib;
use crate::lib::{SensorData, calculate_stress_index, stress_level};

#[derive(Debug, Clone)]
struct EnhancedSensorData {
    data: SensorData,
    stress_index: f64,
    stress_level: String,
}

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
        let use_serial = env::var("USE_SERIAL").unwrap_or_else(|_| "true".to_string()).parse::<bool>().unwrap_or(true);
        let serial_tcp_port = env::var("SERIAL_TCP_PORT").unwrap_or_else(|_| "5555".to_string()).parse::<u16>().unwrap_or(5555);

        Self {
            redis_url: env::var("REDIS_URL").expect("REDIS_URL missing"),
            mysql_url: env::var("MYSQL_DATABASE_URL").expect("MYSQL_DATABASE_URL missing"),
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            use_serial,
            serial_tcp_host: env::var("SERIAL_TCP_HOST").unwrap_or_else(|_| "host.docker.internal".to_string()),
            serial_tcp_port,
        }
    }
}

struct AppState {
    redis: Arc<Mutex<redis::Client>>,
    mysql: Pool,
    memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    config: AppConfig,
}

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

async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
    match TcpStream::connect((host, port)).await {
        Ok(stream) => {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(_) => serde_json::from_str::<SensorData>(&line).ok(),
                Err(e) => { warn!("TCP read error: {:?}", e); None }
            }
        }
        Err(e) => { warn!("TCP connect failed: {:?}", e); None }
    }
}

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
            if mem.len() > 600 { mem.pop_front(); }
        }
    }
}

async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({"status": "healthy", "timestamp": Utc::now()})))
}

async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.memory.lock().await;
    let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
    Ok(HttpResponse::Ok().json(data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    fmt().with_env_filter(EnvFilter::from_default_env()).json().init();

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
