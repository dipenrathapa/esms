#![no_main]
use libfuzzer_sys::fuzz_target;
use esms_backend::business::calculate_stress_index;
use esms_backend::models::SensorData;

fuzz_target!(|data: &[u8]| {
    if data.len() < 32 { return; }

    // Convert bytes to f64
    let temp = f64::from_le_bytes(data[0..8].try_into().unwrap());
    let hum = f64::from_le_bytes(data[8..16].try_into().unwrap());
    let noise = f64::from_le_bytes(data[16..24].try_into().unwrap());
    let hr = f64::from_le_bytes(data[24..32].try_into().unwrap());

    // Filter out NaN or Infinity
    if !temp.is_finite() || !hum.is_finite() || !noise.is_finite() || !hr.is_finite() {
        return; // skip this input
    }

    // Clamp to realistic ranges
    let sensor = SensorData {
        temperature: temp.clamp(-50.0, 60.0),
        humidity: hum.clamp(0.0, 100.0),
        noise: noise.clamp(0.0, 120.0),
        heart_rate: hr.clamp(30.0, 220.0),
        motion: true,
        timestamp: "2026-01-29T00:00:00Z".to_string(),
    };

    let stress = calculate_stress_index(&sensor);

    assert!(stress.is_finite(), "Stress index should never be NaN or Inf");
});
