use crate::models::{EnhancedSensorData, SensorData};
use validator::Validate;

// ============================================================================
// SensorData Validation Tests
// ============================================================================

#[cfg(test)]
mod sensor_data_validation {
    use super::*;

    #[test]
    fn test_valid_sensor_data() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Valid sensor data should pass validation"
        );
    }

    // Temperature validation tests
    #[test]
    fn test_temperature_too_low() {
        let data = SensorData {
            temperature: -0.1,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Temperature below 0 should fail validation"
        );
    }

    #[test]
    fn test_temperature_too_high() {
        let data = SensorData {
            temperature: 60.1,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Temperature above 60 should fail validation"
        );
    }

    #[test]
    fn test_temperature_minimum_boundary() {
        let data = SensorData {
            temperature: 0.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Temperature at 0.0 should be valid"
        );
    }

    #[test]
    fn test_temperature_maximum_boundary() {
        let data = SensorData {
            temperature: 60.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Temperature at 60.0 should be valid"
        );
    }

    // Humidity validation tests
    #[test]
    fn test_humidity_too_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: -0.1,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Humidity below 0 should fail validation"
        );
    }

    #[test]
    fn test_humidity_too_high() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 100.1,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Humidity above 100 should fail validation"
        );
    }

    #[test]
    fn test_humidity_minimum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 0.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Humidity at 0.0 should be valid");
    }

    #[test]
    fn test_humidity_maximum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 100.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Humidity at 100.0 should be valid");
    }

    // Noise validation tests
    #[test]
    fn test_noise_too_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: -0.1,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Noise below 0 should fail validation"
        );
    }

    #[test]
    fn test_noise_too_high() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 120.1,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Noise above 120 should fail validation"
        );
    }

    #[test]
    fn test_noise_minimum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 0.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Noise at 0.0 should be valid");
    }

    #[test]
    fn test_noise_maximum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 120.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Noise at 120.0 should be valid");
    }

    // Heart rate validation tests
    #[test]
    fn test_heart_rate_too_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 29.9,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate below 30 should fail validation"
        );
    }

    #[test]
    fn test_heart_rate_too_high() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 200.1,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate above 200 should fail validation"
        );
    }

    #[test]
    fn test_heart_rate_minimum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 30.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Heart rate at 30.0 should be valid"
        );
    }

    #[test]
    fn test_heart_rate_maximum_boundary() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 200.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Heart rate at 200.0 should be valid"
        );
    }

    // All boundaries at once
    #[test]
    fn test_all_minimum_boundaries() {
        let data = SensorData {
            temperature: 0.0,
            humidity: 0.0,
            noise: 0.0,
            heart_rate: 30.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "All minimum boundaries should be valid"
        );
    }

    #[test]
    fn test_all_maximum_boundaries() {
        let data = SensorData {
            temperature: 60.0,
            humidity: 100.0,
            noise: 120.0,
            heart_rate: 200.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "All maximum boundaries should be valid"
        );
    }

    // Motion field tests
    #[test]
    fn test_motion_true() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Motion true should be valid");
    }

    #[test]
    fn test_motion_false() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(data.validate().is_ok(), "Motion false should be valid");
    }

    // Timestamp tests
    #[test]
    fn test_empty_timestamp() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "".to_string(),
        };

        // Timestamp format is not validated by validator, just structure
        assert!(data.validate().is_ok());
    }

    #[test]
    fn test_various_timestamp_formats() {
        let timestamps = vec![
            "2024-01-01T00:00:00Z",
            "2024-01-01T12:34:56+00:00",
            "2024-01-01 12:34:56",
            "invalid-timestamp",
        ];

        for timestamp in timestamps {
            let data = SensorData {
                temperature: 25.0,
                humidity: 50.0,
                noise: 70.0,
                heart_rate: 75.0,
                motion: false,
                timestamp: timestamp.to_string(),
            };

            // All should pass validation (timestamp format not validated here)
            assert!(data.validate().is_ok());
        }
    }

    // Multiple validation errors
    #[test]
    fn test_multiple_validation_errors() {
        let data = SensorData {
            temperature: -10.0, // Invalid
            humidity: 150.0,    // Invalid
            noise: 200.0,       // Invalid
            heart_rate: 20.0,   // Invalid
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = data.validate();
        assert!(result.is_err(), "Multiple invalid fields should fail");

        if let Err(e) = result {
            // Should have multiple validation errors
            let errors = e.field_errors();
            assert!(errors.len() >= 4, "Should have at least 4 field errors");
        }
    }

    // Floating point precision tests
    #[test]
    fn test_floating_point_precision() {
        let data = SensorData {
            temperature: 25.123456789,
            humidity: 50.987654321,
            noise: 70.111111111,
            heart_rate: 75.555555555,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "High precision floats should be valid"
        );
    }

    #[test]
    fn test_very_small_positive_values() {
        let data = SensorData {
            temperature: 0.0001,
            humidity: 0.0001,
            noise: 0.0001,
            heart_rate: 30.0001,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_ok(),
            "Very small positive values should be valid"
        );
    }
}

// ============================================================================
// EnhancedSensorData Tests
// ============================================================================

#[cfg(test)]
mod enhanced_sensor_data_tests {
    use super::*;

    #[test]
    fn test_enhanced_sensor_data_creation() {
        let sensor_data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let enhanced = EnhancedSensorData {
            data: sensor_data,
            stress_index: 0.345,
            stress_level: "Moderate".to_string(),
        };

        assert_eq!(enhanced.stress_index, 0.345);
        assert_eq!(enhanced.stress_level, "Moderate");
    }

    #[test]
    fn test_enhanced_sensor_data_serialization() {
        let sensor_data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let enhanced = EnhancedSensorData {
            data: sensor_data,
            stress_index: 0.345,
            stress_level: "Moderate".to_string(),
        };

        let json = serde_json::to_string(&enhanced).unwrap();
        assert!(json.contains("temperature"));
        assert!(json.contains("stress_index"));
        assert!(json.contains("stress_level"));
    }

    #[test]
    fn test_enhanced_sensor_data_deserialization() {
        let json = r#"{
            "temperature": 25.0,
            "humidity": 50.0,
            "noise": 70.0,
            "heart_rate": 75.0,
            "motion": false,
            "timestamp": "2024-01-01T00:00:00Z",
            "stress_index": 0.345,
            "stress_level": "Moderate"
        }"#;

        let enhanced: EnhancedSensorData = serde_json::from_str(json).unwrap();
        assert_eq!(enhanced.data.temperature, 25.0);
        assert_eq!(enhanced.stress_index, 0.345);
        assert_eq!(enhanced.stress_level, "Moderate");
    }

    #[test]
    fn test_enhanced_sensor_data_all_stress_levels() {
        let stress_levels = vec!["Low", "Moderate", "High"];

        for level in stress_levels {
            let sensor_data = SensorData {
                temperature: 25.0,
                humidity: 50.0,
                noise: 70.0,
                heart_rate: 75.0,
                motion: false,
                timestamp: "2024-01-01T00:00:00Z".to_string(),
            };

            let enhanced = EnhancedSensorData {
                data: sensor_data,
                stress_index: 0.5,
                stress_level: level.to_string(),
            };

            assert_eq!(enhanced.stress_level, level);
        }
    }

    #[test]
    fn test_enhanced_sensor_data_clone() {
        let sensor_data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let enhanced = EnhancedSensorData {
            data: sensor_data,
            stress_index: 0.345,
            stress_level: "Moderate".to_string(),
        };

        let cloned = enhanced.clone();
        assert_eq!(cloned.stress_index, enhanced.stress_index);
        assert_eq!(cloned.stress_level, enhanced.stress_level);
        assert_eq!(cloned.data.temperature, enhanced.data.temperature);
    }
}

// ============================================================================
// Serialization/Deserialization Edge Cases
// ============================================================================

#[cfg(test)]
mod serialization_tests {
    use super::*;

    #[test]
    fn test_round_trip_serialization() {
        let original = SensorData {
            temperature: 25.123,
            humidity: 50.456,
            noise: 70.789,
            heart_rate: 75.012,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SensorData = serde_json::from_str(&json).unwrap();

        assert_eq!(original.temperature, deserialized.temperature);
        assert_eq!(original.humidity, deserialized.humidity);
        assert_eq!(original.noise, deserialized.noise);
        assert_eq!(original.heart_rate, deserialized.heart_rate);
        assert_eq!(original.motion, deserialized.motion);
        assert_eq!(original.timestamp, deserialized.timestamp);
    }

    #[test]
    fn test_missing_fields_deserialization() {
        let json = r#"{
            "temperature": 25.0,
            "humidity": 50.0,
            "noise": 70.0,
            "heart_rate": 75.0,
            "motion": false
        }"#;

        let result: Result<SensorData, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Missing timestamp should fail");
    }

    #[test]
    fn test_extra_fields_deserialization() {
        let json = r#"{
            "temperature": 25.0,
            "humidity": 50.0,
            "noise": 70.0,
            "heart_rate": 75.0,
            "motion": false,
            "timestamp": "2024-01-01T00:00:00Z",
            "extra_field": "should be ignored"
        }"#;

        let result: Result<SensorData, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "Extra fields should be ignored");
    }
}
