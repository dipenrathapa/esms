#[derive(Debug, Clone)]
pub struct SensorData {
    pub temperature: f64,
    pub humidity: f64,
    pub noise: f64,
    pub heart_rate: f64,
    pub motion: bool,
    pub timestamp: String,
}

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

// Minimal unit test inside lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_math() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_stress_index_low() {
        let data = SensorData {
            temperature: 20.0,
            humidity: 40.0,
            noise: 50.0,
            heart_rate: 60.0,
            motion: false,
            timestamp: "2026-01-26T00:00:00Z".to_string(),
        };
        let index = calculate_stress_index(&data);
        assert!(index >= 0.0 && index <= 1.0);
        assert_eq!(stress_level(index), "Low");
    }

    #[test]
    fn test_stress_index_high() {
        let data = SensorData {
            temperature: 35.0,
            humidity: 80.0,
            noise: 90.0,
            heart_rate: 100.0,
            motion: true,
            timestamp: "2026-01-26T00:00:00Z".to_string(),
        };
        let index = calculate_stress_index(&data);
        assert!(index >= 0.0 && index <= 1.0);
        assert_eq!(stress_level(index), "High");
    }
}
