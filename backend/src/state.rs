use mysql_async::Pool;
use std::{collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::config::AppConfig;
use crate::models::EnhancedSensorData;
use crate::retry::RetryConfig;

pub struct AppState {
    pub redis: Arc<Mutex<redis::Client>>,
    pub mysql: Pool,
    pub memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    pub config: AppConfig,
    #[allow(dead_code)]
    pub shutdown_token: CancellationToken,
    pub retry_config: RetryConfig,
}
