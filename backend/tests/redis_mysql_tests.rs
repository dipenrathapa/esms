use esms_backend::{store_to_mysql, store_to_redis, EnhancedSensorData, SensorData, RetryConfig};
use mysql_async::Pool;
use redis::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_store_to_redis_and_mysql() {
    // Use a test Redis and MySQL instance
    let redis = Arc::new(Mutex::new(Client::open("redis://127.0.0.1/").unwrap()));
    let mysql = Pool::new("mysql://user:pass@localhost/test_db").unwrap();

    let data = EnhancedSensorData {
        data: SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 60.0,
            heart_rate: 75.0,
            motion: true,
            timestamp: "2026-01-27T10:00:00Z".to_string(),
        },
        stress_index: 0.5,
        stress_level: "Moderate".to_string(),
    };

    let retry = RetryConfig::default();

    // Redis
    let redis_result = store_to_redis(redis.clone(), data.clone(), &retry).await;
    assert!(redis_result.is_ok());

    // MySQL
    let mysql_result = store_to_mysql(mysql.clone(), data.clone(), &retry).await;
    assert!(mysql_result.is_ok());
}
