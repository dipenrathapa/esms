use actix_web::{test, web, App};
use esms_backend::lib::{
    calculate_stress_index, simulate_sensor_data, stress_level, EnhancedSensorData, SensorData,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::test]
async fn test_health_endpoint() {
    async fn health() -> actix_web::Result<actix_web::HttpResponse> {
        Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({ "status": "healthy" })))
    }

    let app = test::init_service(App::new().route("/health", web::get().to(health))).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_realtime_endpoint_empty() {
    struct AppStateMock {
        memory: Arc<Mutex<VecDeque<EnhancedSensorData>>>,
    }

    let state = web::Data::new(AppStateMock {
        memory: Arc::new(Mutex::new(VecDeque::new())),
    });

    async fn get_realtime(
        state: web::Data<AppStateMock>,
    ) -> actix_web::Result<actix_web::HttpResponse> {
        let mem = state.memory.lock().await;
        let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
        Ok(actix_web::HttpResponse::Ok().json(data))
    }

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
async fn test_stress_calculation() {
    let data = simulate_sensor_data();
    let index = calculate_stress_index(&data);
    let level = stress_level(index);
    assert!(index >= 0.0 && index <= 1.0);
    assert!(["Low", "Moderate", "High"].contains(&level.as_str()));
}
