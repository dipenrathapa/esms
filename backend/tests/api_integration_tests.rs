use esms_backend::{calculate_stress_index, stress_level, SensorData};

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
fn test_stress_index_moderate() {
    let data = SensorData {
        temperature: 28.0,
        humidity: 60.0,
        noise: 70.0,
        heart_rate: 80.0,
        motion: true,
        timestamp: "2026-01-26T00:00:00Z".to_string(),
    };
    let index = calculate_stress_index(&data);
    assert!(index >= 0.0 && index <= 1.0);
    assert_eq!(stress_level(index), "Moderate");
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
