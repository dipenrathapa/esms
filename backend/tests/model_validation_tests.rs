use esms_backend::models::{EnhancedSensorData, SensorData};
use validator::Validate;
// use validator::ValidationErrors;

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

    // Motion tests
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

        // Borrow the error instead of moving it
        if let Err(ref e) = result {
            assert!(
                e.field_errors().len() >= 4,
                "Should have at least 4 validation errors"
            );
        }
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
}
