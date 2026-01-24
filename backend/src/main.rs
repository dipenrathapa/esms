use actix_web::{web, App, HttpServer, HttpResponse, Result};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use tokio::time::{interval, Duration};

// Sensor data structure matching Arduino output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorData {
    temperature: f64,
    humidity: f64,
    noise: f64,
    heart_rate: f64,
    motion: bool,
    timestamp: String,
}

// FHIR-compatible observation structure
#[derive(Debug, Serialize)]
struct Observation {
    resourceType: String,
    id: String,
    status: String,
    category: Vec<Category>,
    code: Code,
    effectiveDateTime: String,
    valueQuantity: ValueQuantity,
    component: Vec<Component>,
}

#[derive(Debug, Serialize)]
struct Category {
    coding: Vec<Coding>,
}

#[derive(Debug, Serialize)]
struct Coding {
    system: String,
    code: String,
    display: String,
}

#[derive(Debug, Serialize)]
struct Code {
    coding: Vec<Coding>,
    text: String,
}

#[derive(Debug, Serialize)]
struct ValueQuantity {
    value: f64,
    unit: String,
}

#[derive(Debug, Serialize)]
struct Component {
    code: Code,
    valueQuantity: ValueQuantity,
}

// Enhanced response with stress index
#[derive(Debug, Clone, Serialize)]
struct EnhancedSensorData {
    #[serde(flatten)]
    data: SensorData,
    stress_index: f64,
    stress_level: String,
}

// Application state
struct AppState {
    redis_data: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    mysql_data: Arc<Mutex<Vec<EnhancedSensorData>>>,
}

// Calculate stress index
fn calculate_stress_index(data: &SensorData) -> f64 {
    let normalized_hr = (data.heart_rate - 60.0) / 100.0;
    let temp_factor = data.temperature / 50.0;
    let humidity_factor = data.humidity / 100.0;
    let noise_factor = data.noise / 100.0;
    
    (normalized_hr * 0.5) + (temp_factor * 0.2) + (humidity_factor * 0.2) + (noise_factor * 0.1)
}

// Determine stress level
fn get_stress_level(stress_index: f64) -> String {
    if stress_index < 0.3 {
        "Low".to_string()
    } else if stress_index < 0.6 {
        "Moderate".to_string()
    } else {
        "High".to_string()
    }
}

// Convert to FHIR format
fn to_fhir_observation(data: &EnhancedSensorData) -> Observation {
    Observation {
        resourceType: "Observation".to_string(),
        id: format!("stress-{}", data.data.timestamp),
        status: "final".to_string(),
        category: vec![Category {
            coding: vec![Coding {
                system: "http://terminology.hl7.org/CodeSystem/observation-category".to_string(),
                code: "vital-signs".to_string(),
                display: "Vital Signs".to_string(),
            }],
        }],
        code: Code {
            coding: vec![Coding {
                system: "http://loinc.org".to_string(),
                code: "85354-9".to_string(),
                display: "Stress Index".to_string(),
            }],
            text: "Environmental Stress Index".to_string(),
        },
        effectiveDateTime: data.data.timestamp.clone(),
        valueQuantity: ValueQuantity {
            value: data.stress_index,
            unit: "index".to_string(),
        },
        component: vec![
            Component {
                code: Code {
                    coding: vec![Coding {
                        system: "http://loinc.org".to_string(),
                        code: "8310-5".to_string(),
                        display: "Body temperature".to_string(),
                    }],
                    text: "Temperature".to_string(),
                },
                valueQuantity: ValueQuantity {
                    value: data.data.temperature,
                    unit: "Cel".to_string(),
                },
            },
            Component {
                code: Code {
                    coding: vec![Coding {
                        system: "http://loinc.org".to_string(),
                        code: "8867-4".to_string(),
                        display: "Heart rate".to_string(),
                    }],
                    text: "Heart Rate".to_string(),
                },
                valueQuantity: ValueQuantity {
                    value: data.data.heart_rate,
                    unit: "bpm".to_string(),
                },
            },
        ],
    }
}

// Simulate sensor data (for cloud/Codespaces)
fn simulate_sensor_data() -> SensorData {
    use rand::Rng;
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

// Read from Arduino serial port
async fn read_serial_data(port_name: &str) -> Option<SensorData> {
    use tokio_serial::SerialPortBuilderExt;
    
    match tokio_serial::new(port_name, 9600).open_native_async() {
        Ok(mut port) => {
            use tokio::io::AsyncReadExt;
            let mut buf = vec![0u8; 1024];
            
            match port.read(&mut buf).await {
                Ok(n) if n > 0 => {
                    let data_str = String::from_utf8_lossy(&buf[..n]);
                    if let Ok(data) = serde_json::from_str::<SensorData>(&data_str) {
                        return Some(data);
                    }
                }
                _ => {}
            }
        }
        Err(_) => {}
    }
    None
}

// Background task to ingest sensor data
async fn sensor_ingestion_task(state: web::Data<AppState>) {
    let mut interval = interval(Duration::from_secs(1));
    let serial_port = std::env::var("SERIAL_PORT").unwrap_or_else(|_| "/dev/cu.usbmodem113401".to_string());
    
    loop {
        interval.tick().await;
        
        // Try to read from serial, fallback to simulation
        let sensor_data = match read_serial_data(&serial_port).await {
            Some(data) => data,
            None => simulate_sensor_data(),
        };
        
        let stress_index = calculate_stress_index(&sensor_data);
        let stress_level = get_stress_level(stress_index);
        
        let enhanced_data = EnhancedSensorData {
            data: sensor_data,
            stress_index,
            stress_level,
        };
        
        // Store in Redis (last 10 minutes = 600 entries)
        {
            let mut redis = state.redis_data.lock().await;
            redis.push_back(enhanced_data.clone());
            if redis.len() > 600 {
                redis.pop_front();
            }
        }
        
        // Store in MySQL (historical)
        {
            let mut mysql = state.mysql_data.lock().await;
            mysql.push(enhanced_data);
        }
    }
}

// API endpoint: real-time data
async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let redis = state.redis_data.lock().await;
    let recent_data: Vec<EnhancedSensorData> = redis.iter().rev().take(60).cloned().collect();
    
    Ok(HttpResponse::Ok().json(recent_data))
}

// API endpoint: historical data
async fn get_history(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse> {
    let mysql = state.mysql_data.lock().await;
    
    // Simple time filtering (can be enhanced)
    let filtered_data: Vec<EnhancedSensorData> = if let (Some(start), Some(end)) = (query.get("start"), query.get("end")) {
        mysql.iter()
            .filter(|d| d.data.timestamp >= *start && d.data.timestamp <= *end)
            .cloned()
            .collect()
    } else {
        mysql.clone()
    };
    
    Ok(HttpResponse::Ok().json(filtered_data))
}

// API endpoint: FHIR observation
async fn get_fhir_observation(state: web::Data<AppState>) -> Result<HttpResponse> {
    let redis = state.redis_data.lock().await;
    
    if let Some(latest) = redis.back() {
        let observation = to_fhir_observation(latest);
        Ok(HttpResponse::Ok().json(observation))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No data available"
        })))
    }
}

// Health check endpoint
async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339()
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Starting ESMS Backend Server...");
    
    let state = web::Data::new(AppState {
        redis_data: Arc::new(Mutex::new(VecDeque::new())),
        mysql_data: Arc::new(Mutex::new(Vec::new())),
    });
    
    // Start background sensor ingestion
    let state_clone = state.clone();
    tokio::spawn(async move {
        sensor_ingestion_task(state_clone).await;
    });
    
    println!("✅ Backend listening on http://0.0.0.0:8080");
    
    HttpServer::new(move || {
        let cors = Cors::permissive();
        
        App::new()
            .wrap(cors)
            .app_data(state.clone())
            .route("/health", web::get().to(health_check))
            .route("/api/realtime", web::get().to(get_realtime))
            .route("/api/history", web::get().to(get_history))
            .route("/api/fhir/observation", web::get().to(get_fhir_observation))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}