use actix_web::{test, App};
use esms_backend::{
    calculate_stress_index, get_realtime, get_stress_level, health, EnhancedSensorData, SensorData,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_stress_index_low() {
        let data = SensorData {
            temperature: 25.0,
            humidity: 50.0,
            noise: 60.0,
            heart_rate: 60.0,
            motion: false,
            timestamp: "2026-01-24T00:00:00Z".to_string(),
        };
        let stress_index = calculate_stress_index(&data);
        assert!(stress_index < 0.3);
        assert_eq!(get_stress_level(stress_index), "Low");
    }

    #[test]
    fn test_stress_index_high() {
        let data = SensorData {
            temperature: 40.0,
            humidity: 90.0,
            noise: 90.0,
            heart_rate: 120.0,
            motion: true,
            timestamp: "2026-01-24T00:00:00Z".to_string(),
        };
        let stress_index = calculate_stress_index(&data);
        assert!(stress_index > 0.6);
        assert_eq!(get_stress_level(stress_index), "High");
    }
}

// ---------------------- Integration test (API) ----------------------
#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new().route("/health", actix_web::web::get().to(health)),
    )
    .await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_realtime_endpoint() {
    use esms_backend::{redis, AppState};

    let state = actix_web::web::Data::new(AppState {
        redis_client: Arc::new(Mutex::new(
            redis::Client::open("redis://127.0.0.1:6379").unwrap(),
        )),
        mysql_pool: mysql_async::Pool::new("mysql://user:pass@localhost/db"),
        in_memory: Arc::new(Mutex::new(VecDeque::new())),
    });

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .route("/api/realtime", actix_web::web::get().to(get_realtime)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/realtime").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
