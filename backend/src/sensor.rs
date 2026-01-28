// src/sensor.rs
use chrono::Utc;
use rand::Rng;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpStream,
};
use tracing::{error, info, warn};

use crate::models::SensorData;

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

pub async fn read_sensor_from_tcp(host: &str, port: u16) -> Option<SensorData> {
    match TcpStream::connect((host, port)).await {
        Ok(stream) => {
            info!(
                operation = "tcp_connect",
                host = %host,
                port = %port,
                "Successfully connected to TCP sensor stream"
            );

            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(bytes_read) => {
                    if let Ok(sensor) = serde_json::from_str::<SensorData>(&line) {
                        info!(
                            operation = "tcp_read",
                            host = %host,
                            port = %port,
                            bytes_read = %bytes_read,
                            temperature = %sensor.temperature,
                            heart_rate = %sensor.heart_rate,
                            "Successfully parsed sensor data from TCP"
                        );
                        Some(sensor)
                    } else {
                        warn!(
                            operation = "tcp_parse",
                            host = %host,
                            port = %port,
                            raw_data = %line.trim(),
                            "Failed to parse JSON from TCP stream"
                        );
                        None
                    }
                }
                Err(e) => {
                    error!(
                        error = %e,
                        operation = "tcp_read",
                        host = %host,
                        port = %port,
                        "Failed to read data from TCP stream"
                    );
                    None
                }
            }
        }
        Err(e) => {
            error!(
                error = %e,
                operation = "tcp_connect",
                host = %host,
                port = %port,
                "Failed to connect to TCP sensor stream"
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_sensor_data_valid() {
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

    #[test]
    fn test_sensor_data_invalid_temperature_too_low() {
        let data = SensorData {
            temperature: -5.0,
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
    fn test_sensor_data_invalid_temperature_too_high() {
        let data = SensorData {
            temperature: 65.0,
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
    fn test_sensor_data_invalid_humidity() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 105.0,
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
    fn test_sensor_data_invalid_noise() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 125.0,
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
    fn test_sensor_data_invalid_heart_rate_too_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 25.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate below 30 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_invalid_heart_rate_too_high() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 205.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(
            data.validate().is_err(),
            "Heart rate above 200 should fail validation"
        );
    }

    #[test]
    fn test_sensor_data_boundary_values() {
        let min_data = SensorData {
            temperature: 0.0,
            humidity: 0.0,
            noise: 0.0,
            heart_rate: 30.0,
            motion: false,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(
            min_data.validate().is_ok(),
            "Minimum boundary values should be valid"
        );

        let max_data = SensorData {
            temperature: 60.0,
            humidity: 100.0,
            noise: 120.0,
            heart_rate: 200.0,
            motion: true,
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        assert!(
            max_data.validate().is_ok(),
            "Maximum boundary values should be valid"
        );
    }

    #[test]
    fn test_simulate_sensor_data_returns_valid_data() {
        for _ in 0..100 {
            let data = simulate_sensor_data();
            assert!(
                data.validate().is_ok(),
                "Simulated data should always be valid"
            );
        }
    }

    #[test]
    fn test_simulate_sensor_data_ranges() {
        for _ in 0..100 {
            let data = simulate_sensor_data();

            assert!(data.temperature >= 20.0 && data.temperature < 35.0);
            assert!(data.humidity >= 40.0 && data.humidity < 80.0);
            assert!(data.noise >= 50.0 && data.noise < 90.0);
            assert!(data.heart_rate >= 60.0 && data.heart_rate < 100.0);
        }
    }

    #[test]
    fn test_simulate_sensor_data_has_timestamp() {
        let data = simulate_sensor_data();
        assert!(!data.timestamp.is_empty(), "Timestamp should not be empty");
    }
}
