use serde::{Deserialize, Serialize};
use validator::Validate;

/// Sensor data model
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

/// Core business logic
pub fn calculate_stress_index(data: &SensorData) -> f64 {
    let score = (data.heart_rate - 60.0) / 100.0 * 0.5
        + (data.temperature / 50.0) * 0.2
        + (data.humidity / 100.0) * 0.2
        + (data.noise / 100.0) * 0.1;

    score.clamp(0.0, 1.0)
}

/// Stress level categorization
pub fn stress_level(score: f64) -> String {
    match score {
        x if x < 0.3 => "Low",
        x if x < 0.6 => "Moderate",
        _ => "High",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stress_index() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 60.0,
            heart_rate: 80.0,
            motion: false,
            timestamp: "2026-01-26T00:00:00Z".to_string(),
        };
        let score = calculate_stress_index(&data);
        assert!(score >= 0.0 && score <= 1.0);
    }

    #[test]
    fn test_stress_level() {
        assert_eq!(stress_level(0.2), "Low");
        assert_eq!(stress_level(0.5), "Moderate");
        assert_eq!(stress_level(0.8), "High");
    }
}
