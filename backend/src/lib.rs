use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use validator::Validate;

/// ======================================================
/// Models
/// ======================================================
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

/// ======================================================
/// Business Logic
/// ======================================================
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

/// ======================================================
/// Sensor Simulation
/// ======================================================
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
