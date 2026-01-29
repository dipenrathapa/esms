#![no_main]
use libfuzzer_sys::fuzz_target;
use esms_backend::sensor::parse_sensor_data;

fuzz_target!(|data: &[u8]| {
    // Goal: never panic
    let _ = parse_sensor_data(data);
});
