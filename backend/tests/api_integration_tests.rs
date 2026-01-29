use actix_web::{test, web, App};
use chrono::Utc;
use mysql_async::{Opts, Pool};
use redis::Client as RedisClient;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::api::{get_fhir_observation, get_history, get_realtime, get_redis_history, health};
use crate::config::AppConfig;
use crate::models::{EnhancedSensorData, SensorData};
use crate::retry::RetryConfig;
use crate::state::AppState;

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

async fn create_test_app_state() -> web::Data<AppState> {
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

// ============================================================================
// Health Endpoint Tests
// ============================================================================

#[cfg(test)]
mod health_tests {
    use super::*;

    #[actix_web::test]
    async fn test_health_endpoint_returns_ok() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_endpoint_json_structure() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert!(resp.get("status").is_some());
        assert_eq!(resp["status"], "healthy");
        assert!(resp.get("timestamp").is_some());
    }

    #[actix_web::test]
    async fn test_health_endpoint_timestamp_format() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        let timestamp_str = resp["timestamp"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(timestamp_str).is_ok());
    }

    #[actix_web::test]
    async fn test_health_endpoint_multiple_requests() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        for _ in 0..10 {
            let req = test::TestRequest::get().uri("/health").to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success());
        }
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
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_realtime_returns_array() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        assert!(!resp.is_empty());
        assert_eq!(resp.len(), 5);
    }

    #[actix_web::test]
    async fn test_realtime_data_structure() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        let first = &resp[0];
        assert!(first.data.temperature > 0.0);
        assert!(first.data.humidity > 0.0);
        assert!(first.stress_index >= 0.0);
        assert!(!first.stress_level.is_empty());
    }

    #[actix_web::test]
    async fn test_realtime_ordered_newest_first() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        // Verify timestamps are in descending order
        for i in 0..resp.len() - 1 {
            let current = chrono::DateTime::parse_from_rfc3339(&resp[i].data.timestamp).unwrap();
            let next = chrono::DateTime::parse_from_rfc3339(&resp[i + 1].data.timestamp).unwrap();
            assert!(current >= next, "Data should be ordered newest first");
        }
    }

    #[actix_web::test]
    async fn test_realtime_empty_memory() {
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

        let state = web::Data::new(AppState {
            redis: Arc::new(Mutex::new(redis)),
            mysql,
            memory: Arc::new(Mutex::new(VecDeque::new())),
            config,
            shutdown_token: CancellationToken::new(),
            retry_config: RetryConfig::default(),
        });

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp: Vec<EnhancedSensorData> = test::call_and_read_body_json(&app, req).await;

        assert!(resp.is_empty());
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
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/history").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_history_invalid_timestamp_format() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/history?start=invalid&end=also-invalid")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_history_start_after_end() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let end_time = Utc::now();
        let start_time = end_time + chrono::Duration::hours(1);

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/history?start={}&end={}",
                start_time.to_rfc3339(),
                end_time.to_rfc3339()
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_history_valid_rfc3339_timestamps() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::hours(1);

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/history?start={}&end={}",
                start_time.to_rfc3339(),
                end_time.to_rfc3339()
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should return 200 even if no data (connection issues handled gracefully)
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_history_mysql_datetime_format() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/history?start=2024-01-01 00:00:00&end=2024-01-01 23:59:59")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should accept MySQL datetime format
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_history_edge_case_same_start_end() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/history", web::get().to(get_history)),
        )
        .await;

        let time = Utc::now();

        let req = test::TestRequest::get()
            .uri(&format!(
                "/api/history?start={}&end={}",
                time.to_rfc3339(),
                time.to_rfc3339()
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Same start and end should fail validation
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
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        assert_eq!(resp["resourceType"], "Observation");
        assert!(resp.get("id").is_some());
        assert_eq!(resp["status"], "final");
        assert!(resp.get("code").is_some());
        assert!(resp.get("component").is_some());
    }

    #[actix_web::test]
    async fn test_fhir_observation_components() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        let components = resp["component"].as_array().unwrap();
        assert_eq!(components.len(), 5); // temperature, humidity, heart_rate, noise, motion
    }

    #[actix_web::test]
    async fn test_fhir_observation_loinc_codes() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        // Verify main observation has LOINC code
        assert_eq!(resp["code"]["coding"][0]["system"], "http://loinc.org");
        assert_eq!(resp["code"]["coding"][0]["code"], "85354-9");
    }

    #[actix_web::test]
    async fn test_fhir_observation_stress_interpretation() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        let interpretation = &resp["interpretation"][0];
        assert!(interpretation.get("coding").is_some());
        assert!(interpretation.get("text").is_some());
    }

    #[actix_web::test]
    async fn test_fhir_observation_no_data() {
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

        let state = web::Data::new(AppState {
            redis: Arc::new(Mutex::new(redis)),
            mysql,
            memory: Arc::new(Mutex::new(VecDeque::new())),
            config,
            shutdown_token: CancellationToken::new(),
            retry_config: RetryConfig::default(),
        });

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_fhir_observation_effective_datetime() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp: serde_json::Value = test::call_and_read_body_json(&app, req).await;

        let effective_datetime = resp["effectiveDateTime"].as_str().unwrap();
        assert!(chrono::DateTime::parse_from_rfc3339(effective_datetime).is_ok());
    }
}

// ============================================================================
// Integration Tests - Full Workflow
// ============================================================================

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[actix_web::test]
    async fn test_full_api_workflow() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/health", web::get().to(health))
                .route("/api/realtime", web::get().to(get_realtime))
                .route("/api/fhir/observation", web::get().to(get_fhir_observation)),
        )
        .await;

        // 1. Check health
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 2. Get realtime data
        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // 3. Get FHIR observation
        let req = test::TestRequest::get()
            .uri("/api/fhir/observation")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_concurrent_requests() {
        let state = create_test_app_state().await;
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", web::get().to(get_realtime)),
        )
        .await;

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let app = app.clone();
                tokio::spawn(async move {
                    let req = test::TestRequest::get().uri("/api/realtime").to_request();
                    test::call_service(&app, req).await
                })
            })
            .collect();

        for handle in handles {
            let resp = handle.await.unwrap();
            assert!(resp.status().is_success());
        }
    }
}