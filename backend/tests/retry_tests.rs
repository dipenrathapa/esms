use esms_backend::retry::{retry_with_backoff, RetryConfig};
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
                        Err::<i32, &str>("temporary failure")
                    } else {
                        Ok::<i32, &str>(42)
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
                    Err::<i32, &str>("permanent failure")
                }
            },
            &config,
            "test_operation",
        )
        .await;

        assert!(result.is_err());
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
                        Err::<(), &str>("retry")
                    } else {
                        Ok::<(), &str>(())
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 70);
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
                        Err::<(), &str>("retry")
                    } else {
                        Ok::<(), &str>(())
                    }
                }
            },
            &config,
            "test_operation",
        )
        .await;

        let elapsed = start.elapsed();
        assert!(result.is_ok());
        assert!(elapsed.as_millis() >= 550);
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
                    Err::<i32, &str>("failure")
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
    async fn test_retry_different_error_types() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };

        let result = retry_with_backoff(
            || async { Err::<(), String>("error".to_string()) },
            &config,
            "test_operation",
        )
        .await;
        assert!(result.is_err());

        let result = retry_with_backoff(
            || async { Err::<(), &str>("error") },
            &config,
            "test_operation",
        )
        .await;
        assert!(result.is_err());
    }
}
