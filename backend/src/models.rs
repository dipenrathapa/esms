// use serde::{Deserialize, Serialize};
// use validator::Validate;

// #[derive(Debug, Clone, Serialize, Deserialize, Validate)]
// pub struct SensorData {
//     #[validate(range(min = 0.0, max = 60.0))]
//     pub temperature: f64,
//     #[validate(range(min = 0.0, max = 100.0))]
//     pub humidity: f64,
//     #[validate(range(min = 0.0, max = 120.0))]
//     pub noise: f64,
//     #[validate(range(min = 30.0, max = 200.0))]
//     pub heart_rate: f64,
//     pub motion: bool,
//     pub timestamp: String,
// }

// #[derive(Debug, Clone, Serialize)]
// pub struct EnhancedSensorData {
//     #[serde(flatten)]
//     pub data: SensorData,
//     pub stress_index: f64,
//     pub stress_level: String,
// }

use serde::{Deserialize, Serialize};
use validator::Validate;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSensorData {
    #[serde(flatten)]
    pub data: SensorData,
    pub stress_index: f64,
    pub stress_level: String,
}

