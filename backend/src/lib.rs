#[allow(unused_imports)]
use chrono::Utc; // currently unused, but allowed
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use validator::Validate;

// ============================
// Models
// ============================

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SensorData {
    pub temperature: f64,
    pub humidity: f64,
    pub noise: f64,
    pub heart_rate: f64,
    pub motion: bool,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnhancedSensorData {
    pub data: SensorData,
    pub stress_index: f64,
    pub stress_level: String,
}

// ============================
// Functions
// ============================

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

// ============================
// AppState
// ============================

pub struct AppState {
    pub memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
}

// ============================
// Helper to create AppState
// ============================

impl AppState {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

// Implement Default to satisfy Clippy/CI
impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
