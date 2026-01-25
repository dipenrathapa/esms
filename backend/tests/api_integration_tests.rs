use actix_web::{test, web, App};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use crate as esms_backend_main; 

// Bring in everything from main.rs
use esms_backend_main::{
    calculate_stress_index, get_realtime, health, SensorData, EnhancedSensorData,
    AppState, AppConfig, stress_level,
};

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn stress_index_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 60.0,
            heart_rate: 60.0,
            motion: false,
            timestamp: "2026-01-24T00:00:00Z".to_string(),
        };
        let score = calculate_stress_index(&data);
        assert!(score < 0.3);
        assert_eq!(stress_level(score), "Low");
    }

    #[test]
    fn stress_index_high() {
        let data = SensorData {
            temperature: 40.0,
            humidity: 90.0,
            noise: 90.0,
            heart_rate: 120.0,
            motion: true,
            timestamp: "2026-01-24T00:00:00Z".to_string(),
        };
        let score = calculate_stress_index(&data);
        assert!(score > 0.6);
        assert_eq!(stress_level(score), "High");
    }
}

// ---------------------- Integration tests ----------------------
#[cfg(test)]
mod integration_tests {
    use super::*;
    use redis::AsyncCommands;
    use tokio::time::sleep;

    /// Setup AppState connecting to local Redis & MySQL
    async fn setup_app_state() -> web::Data<AppState> {
        let config = AppConfig::from_env();

        // Retry loop for Redis
        let redis_client = loop {
            match redis::Client::open(config.redis_url.clone()) {
                Ok(client) => break client,
                Err(_) => sleep(Duration::from_secs(1)).await,
            }
        };

        let mysql_pool = mysql_async::Pool::new(config.mysql_url.clone());

        web::Data::new(AppState {
            redis: Arc::new(Mutex::new(redis_client)),
            mysql: mysql_pool,
            memory: Arc::new(Mutex::new(VecDeque::new())),
            config,
        })
    }

    /// Health endpoint test
    #[actix_web::test]
    async fn health_endpoint() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    /// Realtime API test (requires Redis & MySQL)
    /// Ignored by default for local dev
    #[actix_web::test]
    #[ignore]
    async fn get_realtime_endpoint() {
        let state = setup_app_state().await;

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body_bytes = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body_bytes).unwrap_or("");
        assert!(body_str.starts_with('[') && body_str.ends_with(']'));
    }
}
