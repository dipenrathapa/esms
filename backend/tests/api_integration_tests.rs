use actix_web::{test, App, web};
use esms_backend::lib::{
    simulate_sensor_data, calculate_stress_index, stress_level,
    AppState, AppConfig, EnhancedSensorData, get_realtime, health,
};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().route("/health", web::get().to(health))).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_realtime_endpoint_empty() {
    let memory: Arc<Mutex<VecDeque<EnhancedSensorData>>> = Arc::new(Mutex::new(VecDeque::new()));

    let state = web::Data::new(AppState {
        memory: memory.clone(),
        config: AppConfig {
            redis_url: "".to_string(),
            mysql_url: "".to_string(),
            bind_addr: "0.0.0.0:8080".to_string(),
            use_serial: false,
            serial_tcp_host: "".to_string(),
            serial_tcp_port: 5555,
        },
        redis: Arc::new(Mutex::new(redis::Client::open("redis://127.0.0.1/").unwrap())),
        mysql: mysql_async::Pool::new("mysql://root:root@localhost:3306/test"),
    });

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .route("/api/realtime", web::get().to(get_realtime))
    ).await;

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
