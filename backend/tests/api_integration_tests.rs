use actix_web::{test, App};
use esms_backend::{calculate_stress_index, get_stress_level, health, get_realtime, SensorData, EnhancedSensorData};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

// ---------------------- Unit tests ----------------------
#[actix_rt::test]
async fn test_calculate_stress_index_and_level() {
    let data = SensorData {
        temperature: 30.0,
        humidity: 50.0,
        noise: 70.0,
        heart_rate: 80.0,
        motion: true,
        timestamp: "2026-01-24T00:00:00Z".to_string(),
    };
    let index = calculate_stress_index(&data);
    let level = get_stress_level(index);

    // Stress index should be between 0 and 1
    assert!(index >= 0.0 && index <= 1.0);

    // Stress level should be one of the expected strings
    assert!(level == "Low" || level == "Moderate" || level == "High");
}

// ---------------------- Integration test (API) ----------------------
#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().route("/health", actix_web::web::get().to(health))).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_realtime_endpoint() {
    // Setup dummy AppState
    use esms_backend::{AppState, redis};
    let state = actix_web::web::Data::new(AppState {
        redis_client: Arc::new(Mutex::new(redis::Client::open("redis://127.0.0.1:6379").unwrap())),
        mysql_pool: mysql_async::Pool::new("mysql://root:password@127.0.0.1:3306/esms_db"),
        in_memory: Arc::new(Mutex::new(VecDeque::new())),
    });

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .route("/api/realtime", actix_web::web::get().to(get_realtime))
    ).await;

    let req = test::TestRequest::get().uri("/api/realtime").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}
