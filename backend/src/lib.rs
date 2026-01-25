// #[allow(unused_imports)]
// use chrono::Utc; // currently unused, but allowed
// use serde::{Deserialize, Serialize};
// use std::collections::VecDeque;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use validator::Validate;

// // ============================
// // Models
// // ============================

// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// pub struct SensorData {
//     pub temperature: f64,
//     pub humidity: f64,
//     pub noise: f64,
//     pub heart_rate: f64,
//     pub motion: bool,
//     pub timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// pub struct EnhancedSensorData {
//     pub data: SensorData,
//     pub stress_index: f64,
//     pub stress_level: String,
// }

// // ============================
// // Functions
// // ============================

// pub fn calculate_stress_index(data: &SensorData) -> f64 {
//     let score = (data.heart_rate - 60.0) / 100.0 * 0.5
//         + (data.temperature / 50.0) * 0.2
//         + (data.humidity / 100.0) * 0.2
//         + (data.noise / 100.0) * 0.1;

//     score.clamp(0.0, 1.0)
// }

// pub fn stress_level(score: f64) -> String {
//     match score {
//         x if x < 0.3 => "Low",
//         x if x < 0.6 => "Moderate",
//         _ => "High",
//     }
//     .to_string()
// }

// // ============================
// // AppState
// // ============================

// pub struct AppState {
//     pub memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
// }

// // ============================
// // Helper to create AppState
// // ============================

// impl AppState {
//     pub fn new() -> Self {
//         Self {
//             memory: Arc::new(Mutex::new(VecDeque::new())),
//         }
//     }
// }

// // Implement Default to satisfy Clippy/CI
// impl Default for AppState {
//     fn default() -> Self {
//         Self::new()
//     }
// }



use chrono::Utc;
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
    pub data: SensorData,
    pub stress_index: f64,
    pub stress_level: String,
}

// ============================
// Business Logic
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
// App State
// ============================

pub struct AppState {
    pub memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            memory: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================
// UNIT TESTS
// ============================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sensor_data(
        temp: f64,
        humidity: f64,
        noise: f64,
        hr: f64,
        motion: bool,
    ) -> SensorData {
        SensorData {
            temperature: temp,
            humidity,
            noise,
            heart_rate: hr,
            motion,
            timestamp: Utc::now().to_rfc3339(),
        }
    }

    #[test]
    fn test_stress_calculation_low() {
        let data = create_test_sensor_data(20.0, 40.0, 50.0, 60.0, false);
        let stress = calculate_stress_index(&data);
        assert!(stress < 0.3, "Expected low stress, got {}", stress);
        assert_eq!(stress_level(stress), "Low");
    }

    #[test]
    fn test_stress_calculation_moderate() {
        let data = create_test_sensor_data(28.0, 60.0, 70.0, 80.0, false);
        let stress = calculate_stress_index(&data);
        assert!(
            stress >= 0.3 && stress < 0.6,
            "Expected moderate stress, got {}",
            stress
        );
        assert_eq!(stress_level(stress), "Moderate");
    }

    #[test]
    fn test_stress_calculation_high() {
        let data = create_test_sensor_data(35.0, 80.0, 90.0, 120.0, true);
        let stress = calculate_stress_index(&data);
        assert!(stress >= 0.6, "Expected high stress, got {}", stress);
        assert_eq!(stress_level(stress), "High");
    }

    #[test]
    fn test_validation_valid_data() {
        let data = create_test_sensor_data(25.0, 50.0, 60.0, 75.0, false);
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_validation_invalid_temperature() {
        let data = create_test_sensor_data(70.0, 50.0, 60.0, 75.0, false);
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_heart_rate() {
        let data = create_test_sensor_data(25.0, 50.0, 60.0, 250.0, false);
        assert!(data.validate().is_err());
    }

    #[test]
    fn test_stress_index_bounds() {
        // Test extreme low
        let low_data = create_test_sensor_data(0.0, 0.0, 0.0, 30.0, false);
        let low_stress = calculate_stress_index(&low_data);
        assert!(low_stress >= 0.0 && low_stress <= 1.0);

        // Test extreme high
        let high_data = create_test_sensor_data(60.0, 100.0, 120.0, 200.0, true);
        let high_stress = calculate_stress_index(&high_data);
        assert!(high_stress >= 0.0 && high_stress <= 1.0);
    }

    #[test]
    fn test_stress_level_boundaries() {
        assert_eq!(stress_level(0.0), "Low");
        assert_eq!(stress_level(0.29), "Low");
        assert_eq!(stress_level(0.3), "Moderate");
        assert_eq!(stress_level(0.5), "Moderate");
        assert_eq!(stress_level(0.59), "Moderate");
        assert_eq!(stress_level(0.6), "High");
        assert_eq!(stress_level(1.0), "High");
    }

    #[test]
    fn test_sensor_data_serialization() {
        let data = create_test_sensor_data(25.0, 50.0, 60.0, 75.0, false);
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: SensorData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(data.temperature, deserialized.temperature);
        assert_eq!(data.humidity, deserialized.humidity);
        assert_eq!(data.motion, deserialized.motion);
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let state = AppState::new();
        let memory = state.memory.lock().await;
        assert_eq!(memory.len(), 0);
    }
}

// ============================
// PROPERTY-BASED TESTS
// ============================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn stress_index_always_in_range(
            temp in 0.0f64..60.0,
            humidity in 0.0f64..100.0,
            noise in 0.0f64..120.0,
            hr in 30.0f64..200.0,
        ) {
            let data = SensorData {
                temperature: temp,
                humidity,
                noise,
                heart_rate: hr,
                motion: false,
                timestamp: Utc::now().to_rfc3339(),
            };
            
            let stress = calculate_stress_index(&data);
            prop_assert!(stress >= 0.0 && stress <= 1.0);
        }

        #[test]
        fn stress_level_is_consistent(score in 0.0f64..1.0) {
            let level = stress_level(score);
            prop_assert!(level == "Low" || level == "Moderate" || level == "High");
        }
    }
}