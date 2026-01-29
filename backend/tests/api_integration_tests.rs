use actix_web::{test, web, App};
use chrono::Utc;
use futures::future::join_all;
use mysql_async::{Opts, Pool};
use redis::Client as RedisClient;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use esms_backend::api::{get_fhir_observation, get_history, get_realtime, health};
use esms_backend::config::AppConfig;
use esms_backend::models::{EnhancedSensorData, SensorData};
use esms_backend::retry::RetryConfig;
use esms_backend::state::AppState;

// ============================================================================
// Test Helpers & Utilities
// ============================================================================

fn create_mock_sensor_data(timestamp: &str) -> EnhancedSensorData {
    EnhancedSensorData {
        data: SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 70.0,
            heart_rate: 75.0,
            motion: false,
            timestamp: timestamp.to_string(),
        },
        stress_index: 0.345,
        stress_level: "Moderate".to_string(),
    }
}

fn create_test_app_state() -> web::Data<AppState> {
    let config = AppConfig {
        redis_url: "redis://localhost:6379".to_string(),
        mysql_url: "mysql://root:password@localhost:3306/test_db".to_string(),
        bind_addr: "127.0.0.1:8080".to_string(),
        use_serial: false,
        serial_tcp_host: "localhost".to_string(),
        serial_tcp_port: 5555,
    };

    let redis = RedisClient::open(config.redis_url.clone()).unwrap();
    let mysql = Pool::new(Opts::from_url(&config.mysql_url).unwrap());

    let mut memory = VecDeque::new();
    for i in 0..5 {
        let timestamp = Utc::now()
            .checked_sub_signed(chrono::Duration::seconds(i))
            .unwrap()
            .to_rfc3339();
        memory.push_back(create_mock_sensor_data(&timestamp));
    }

    web::Data::new(AppState {
        redis: Arc::new(Mutex::new(redis)),
        mysql,
        memory: Arc::new(Mutex::new(memory)),
        config,
        shutdown_token: CancellationToken::new(),
        retry_config: RetryConfig::default(),
    })
}

/// Initialize Actix App with all routes
fn init_test_app(state: web::Data<AppState>) -> App<()> {
    App::new()
        .app_data(state)
        .route("/health", web::get().to(health))
        .route("/api/realtime", web::get().to(get_realtime))
        .route("/api/history", web::get().to(get_history))
        .route("/api/fhir/observation", web::get().to(get_fhir_observation))
}

// ============================================================================
// Health Endpoint Tests
// ============================================================================

#[cfg(test)]
mod health_tests {
    use super::*;

    #[actix_web::test]
    async fn test_health_endpoint_returns_ok() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_endpoint_json_structure() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["status"], "healthy");
        assert!(resp.get("timestamp").is_some());
    }

    #[actix_web::test]
    async fn test_health_endpoint_timestamp_format() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        let timestamp_str = resp["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_str).is_ok());
    }
}

// ============================================================================
// Realtime API Tests
// ============================================================================

#[cfg(test)]
mod realtime_tests {
    use super::*;

    #[actix_web::test]
    async fn test_realtime_returns_data() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        assert!(!resp.is_empty());
        assert_eq!(resp.len(), 5);
    }

    #[actix_web::test]
    async fn test_realtime_ordered_newest_first() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        for i in 0..resp.len() - 1 {
            let current = chrono::DateTime::parse_from_rfc3339(&resp[i].data.timestamp).unwrap();
            let next = chrono::DateTime::parse_from_rfc3339(&resp[i + 1].data.timestamp).unwrap();
            assert!(current >= next);
        }
    }
}

// ============================================================================
// History API Tests
// ============================================================================

#[cfg(test)]
mod history_tests {
    use super::*;

    #[actix_web::test]
    async fn test_history_missing_parameters() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/api/history").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }
}

// ============================================================================
// FHIR Observation Tests
// ============================================================================

#[cfg(test)]
mod fhir_tests {
    use super::*;

    #[actix_web::test]
    async fn test_fhir_observation_structure() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let req = test::TestRequest::get().uri("/api/fhir/observation").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["resourceType"], "Observation");
    }
}

// ============================================================================
// Integration & Workflow Tests
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[actix_web::test]
    async fn test_full_api_workflow() {
        let state = create_test_app_state();
        let app = test::init_service(init_test_app(state)).await;

        let health_resp = test::call_service(&app, test::TestRequest::get().uri("/health").to_request()).await;
        assert!(health_resp.status().is_success());

        let realtime_resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, test::TestRequest::get().uri("/api/realtime").to_request()).await;
        assert!(!realtime_resp.is_empty());

        let fhir_resp: serde_json::Value = test::call_and_read_body_json(&app, test::TestRequest::get().uri("/api/fhir/observation").to_request()).await;
        assert_eq!(fhir_resp["resourceType"], "Observation");
    }
}
