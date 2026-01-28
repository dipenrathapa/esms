// src/business.rs
use crate::models::SensorData;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_index_returns_value_in_range() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(
            (0.0..=1.0).contains(&index),
            "Stress index should be between 0 and 1"
        );
    }

    #[test]
    fn test_stress_index_minimum_values() {
        let data = SensorData {
            temperature: 0.0,
            humidity: 0.0,
            noise: 0.0,
            heart_rate: 60.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(index < 0.1, "Minimum stress should be very low");
    }

    #[test]
    fn test_stress_index_maximum_values() {
        let data = SensorData {
            temperature: 50.0,
            humidity: 100.0,
            noise: 100.0,
            heart_rate: 160.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(index > 0.5, "High vitals should produce high stress index");
    }

    #[test]
    fn test_stress_index_moderate_values() {
        let data = SensorData {
            temperature: 30.0,
            humidity: 60.0,
            noise: 70.0,
            heart_rate: 90.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let index = calculate_stress_index(&data);
        assert!(
            (0.3..=0.7).contains(&index),
            "Moderate values should produce moderate stress"
        );
    }

    #[test]
    fn test_stress_level_low() {
        assert_eq!(stress_level(0.0), "Low");
        assert_eq!(stress_level(0.1), "Low");
        assert_eq!(stress_level(0.29), "Low");
    }

    #[test]
    fn test_stress_level_moderate() {
        assert_eq!(stress_level(0.3), "Moderate");
        assert_eq!(stress_level(0.45), "Moderate");
        assert_eq!(stress_level(0.59), "Moderate");
    }

    #[test]
    fn test_stress_level_high() {
        assert_eq!(stress_level(0.6), "High");
        assert_eq!(stress_level(0.8), "High");
        assert_eq!(stress_level(1.0), "High");
    }

    #[test]
    fn test_stress_level_boundary_conditions() {
        assert_eq!(stress_level(0.29999), "Low");
        assert_eq!(stress_level(0.30000), "Moderate");
        assert_eq!(stress_level(0.59999), "Moderate");
        assert_eq!(stress_level(0.60000), "High");
    }
}