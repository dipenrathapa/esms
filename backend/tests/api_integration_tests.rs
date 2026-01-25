use esms_backend::{calculate_stress_index, stress_level, SensorData};
use tokio;

#[tokio::test]
async fn stress_index_low() {
    let data = SensorData {
        temperature: 20.0,
        humidity: 40.0,
        noise: 50.0,
        heart_rate: 60.0,
        motion: false,
        timestamp: "2026-01-25T00:00:00Z".into(),
    };

    let index = calculate_stress_index(&data);
    let level = stress_level(index);

    assert_eq!(level, "Low");
    assert!(index >= 0.0 && index <= 0.3);
}

#[tokio::test]
async fn stress_index_high() {
    let data = SensorData {
        temperature: 45.0,
        humidity: 90.0,
        noise: 90.0,
        heart_rate: 120.0,
        motion: true,
        timestamp: "2026-01-25T00:00:00Z".into(),
    };

    let index = calculate_stress_index(&data);
    let level = stress_level(index);

    assert_eq!(level, "High");
    assert!(index > 0.6);
}
