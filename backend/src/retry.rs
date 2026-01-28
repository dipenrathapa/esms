// src/retry.rs
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, Fut, T, E>(
    operation: F,
    config: &RetryConfig,
    operation_name: &str,
) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay_ms;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    info!(
                        operation = operation_name,
                        attempts = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(e) if attempt >= config.max_attempts => {
                error!(
                    operation = operation_name,
                    attempts = attempt,
                    error = %e,
                    "Operation failed after maximum retry attempts"
                );
                return Err(e);
            }
            Err(e) => {
                warn!(
                    operation = operation_name,
                    attempt = attempt,
                    max_attempts = config.max_attempts,
                    delay_ms = delay,
                    error = %e,
                    "Operation failed, retrying..."
                );

                sleep(Duration::from_millis(delay)).await;
                #[allow(
                    clippy::cast_possible_truncation,
                    clippy::cast_sign_loss,
                    clippy::cast_precision_loss
                )]
                {
                    delay = ((delay as f64 * config.multiplier) as u64).min(config.max_delay_ms);
                }
            }
        }
    }
}