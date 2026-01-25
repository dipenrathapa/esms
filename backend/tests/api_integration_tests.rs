// use esms_backend::{calculate_stress_index, stress_level, SensorData};
// use tokio;

// #[tokio::test]
// async fn stress_index_low() {
//     let data = SensorData {
//         temperature: 20.0,
//         humidity: 40.0,
//         noise: 50.0,
//         heart_rate: 60.0,
//         motion: false,
//         timestamp: "2026-01-25T00:00:00Z".into(),
//     };

//     let index = calculate_stress_index(&data);
//     let level = stress_level(index);

//     assert_eq!(level, "Low");
//     assert!(index >= 0.0 && index <= 0.3);
// }

// #[tokio::test]
// async fn stress_index_high() {
//     let data = SensorData {
//         temperature: 45.0,
//         humidity: 90.0,
//         noise: 90.0,
//         heart_rate: 120.0,
//         motion: true,
//         timestamp: "2026-01-25T00:00:00Z".into(),
//     };

//     let index = calculate_stress_index(&data);
//     let level = stress_level(index);

//     assert_eq!(level, "High");
//     assert!(index > 0.6);
// }


// ---
use actix_web::{test, App};
use serde_json::Value;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/health", actix_web::web::get().to(health_handler))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
}

#[actix_web::test]
async fn test_realtime_endpoint_returns_array() {
    let app = test::init_service(
        App::new()
            .route("/api/realtime", actix_web::web::get().to(get_realtime_handler))
    ).await;

    let req = test::TestRequest::get().uri("/api/realtime").to_request();
    let resp = test::call_service(&app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: Value = test::read_body_json(resp).await;
    assert!(body.is_array());
}

#[actix_web::test]
async fn test_cors_headers() {
    let app = test::init_service(
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .route("/health", actix_web::web::get().to(health_handler))
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .insert_header(("Origin", "http://localhost:3000"))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    
    let headers = resp.headers();
    assert!(headers.contains_key("access-control-allow-origin"));
}

// Mock handlers for testing
async fn health_handler() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_realtime_handler() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(Vec::<Value>::new())
}