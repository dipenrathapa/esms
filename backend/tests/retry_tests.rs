use crate::retry::{retry_with_backoff, RetryConfig};
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

#[cfg(test)]
mod retry_tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_succeeds_first_attempt() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || async { Ok::<i32, String>(42) },
            &config,
            "test_operation",
        )
        .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err("temporary failure")
                    } else {
                        Ok(42)
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_after_max_attempts() {
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("permanent failure")
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "permanent failure");
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 4,
            initial_delay_ms: 10,
            max_delay_ms: 1000,
            multiplier: 2.0,
        };

        let start = std::time::Instant::now();
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 3 {
                        Err("retry")
                    } else {
                        Ok(())
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Should have delays: 10ms, 20ms, 40ms = 70ms minimum
        assert!(
            elapsed.as_millis() >= 70,
            "Expected at least 70ms, got {}ms",
            elapsed.as_millis()
        );
    }

    #[tokio::test]
    async fn test_retry_max_delay_cap() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 150,
            multiplier: 2.0,
        };

        let start = std::time::Instant::now();
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count < 4 {
                        Err("retry")
                    } else {
                        Ok(())
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Delays should be: 100ms, 150ms (capped), 150ms (capped), 150ms (capped)
        // Total minimum: 550ms
        assert!(
            elapsed.as_millis() >= 550,
            "Expected at least 550ms, got {}ms",
            elapsed.as_millis()
        );
    }

    #[tokio::test]
    async fn test_retry_single_attempt() {
        let config = RetryConfig {
            max_attempts: 1,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let result = retry_with_backoff(
            || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("failure")
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(attempt_counter.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_config_default() {
        let config = RetryConfig::default();

        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 5000);
        assert_eq!(config.multiplier, 2.0);
    }

    #[tokio::test]
    async fn test_retry_different_error_types() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        // Test with String error
        let result = retry_with_backoff(
            || async { Err::<(), String>("error".to_string()) },
            &config,
            "test_operation",
        )
        .await;
        assert!(result.is_err());

        // Test with &str error
        let result = retry_with_backoff(
            || async { Err::<(), &str>("error") },
            &config,
            "test_operation",
        )
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_preserves_success_value() {
        let config = RetryConfig::default();

        #[derive(Debug, PartialEq)]
        struct ComplexType {
            value: i32,
            text: String,
        }

        let expected = ComplexType {
            value: 42,
            text: "test".to_string(),
        };

        let result = retry_with_backoff(
            || {
                let data = ComplexType {
                    value: 42,
                    text: "test".to_string(),
                };
                async move { Ok::<ComplexType, String>(data) }
            },
            &config,
            "test_operation",
        )
        .await;

        assert_eq!(result.unwrap(), expected);
    }
}

#[cfg(test)]
mod retry_config_tests {
    use super::*;

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_delay_ms: 50,
            max_delay_ms: 2000,
            multiplier: 1.5,
        };

        let cloned = config.clone();

        assert_eq!(cloned.max_attempts, config.max_attempts);
        assert_eq!(cloned.initial_delay_ms, config.initial_delay_ms);
        assert_eq!(cloned.max_delay_ms, config.max_delay_ms);
        assert_eq!(cloned.multiplier, config.multiplier);
    }

    #[test]
    fn test_retry_config_custom_values() {
        let config = RetryConfig {
            max_attempts: 7,
            initial_delay_ms: 200,
            max_delay_ms: 10000,
            multiplier: 3.0,
        };

        assert_eq!(config.max_attempts, 7);
        assert_eq!(config.initial_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 10000);
        assert_eq!(config.multiplier, 3.0);
    }

    #[test]
    fn test_retry_config_edge_cases() {
        let config = RetryConfig {
            max_attempts: 1,
            initial_delay_ms: 1,
            max_delay_ms: 1,
            multiplier: 1.0,
        };

        assert_eq!(config.max_attempts, 1);
        assert_eq!(config.initial_delay_ms, 1);
        assert_eq!(config.max_delay_ms, 1);
        assert_eq!(config.multiplier, 1.0);
    }
}
