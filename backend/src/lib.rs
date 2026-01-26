use chrono::Utc;
use mysql_async::{Pool, prelude::Queryable};
use rand::Rng;
use redis::Client;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use validator::Validate;

// ======================================================
// Models
// ======================================================

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SensorData {
    #[validate(range(min = 0.0, max = 60.0))]
    pub temperature: f64,

    #[validate(range(min = 0.0, max = 100.0))]
    pub humidity: f64,

    #[validate(range(min = 0.0, max = 120.0))]
    pub noise: f64,

    #[validate(range(min = 30.0, max = 200.0))]
    pub heart_rate: f64,

    pub motion: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnhancedSensorData {
    #[serde(flatten)]
    pub data: SensorData,
    pub stress_index: f64,
    pub stress_level: String,
}

// ======================================================
// App State & Config
// ======================================================

#[derive(Clone)]
pub struct AppConfig {
    pub redis_url: String,
    pub mysql_url: String,
    pub bind_addr: String,
    pub use_serial: bool,
    pub serial_tcp_host: String,
    pub serial_tcp_port: u16,
}

pub struct AppState {
    pub redis: Arc<Mutex<Client>>,
    pub mysql: Pool,
    pub memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    pub config: AppConfig,
}

// ======================================================
// Business Logic
// ======================================================

pub fn calculate_stress_index(data: &SensorData) -> f64 {
    let score = (data.heart_rate - 60.0) / 100.0 * 0.5
        + (data.temperature / 50.0) * 0.2
        + (data.humidity / 100.0) * 0.2
        + (data.noise / 100.0) * 0.1;

    score.clamp(0.0, 1.0)
}

pub fn stress_level(score: f64) -> String {
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

pub fn simulate_sensor_data() -> SensorData {
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
// Helper for accessing in-memory data
// ======================================================

pub async fn get_latest_memory(
    state: &AppState,
) -> Vec<EnhancedSensorData> {
    let mem = state.memory.lock().await; // use tokio::sync::MutexGuard
    mem.iter().rev().take(60).cloned().collect()
}
