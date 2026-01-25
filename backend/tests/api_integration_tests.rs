// use actix_web::{test, App};
// use esms_backend::{
//     calculate_stress_index, get_realtime, get_stress_level, health, EnhancedSensorData, SensorData,
// };
// use std::collections::VecDeque;
// use std::sync::Arc;
// use tokio::sync::Mutex;

// // ---------------------- Unit tests ----------------------
// #[cfg(test)]
// mod unit_tests {
//     use super::*;

//     #[test]
//     fn test_stress_index_low() {
//         let data = SensorData {
//             temperature: 25.0,
//             humidity: 50.0,
//             noise: 60.0,
//             heart_rate: 60.0,
//             motion: false,
//             timestamp: "2026-01-24T00:00:00Z".to_string(),
//         };
//         let stress_index = calculate_stress_index(&data);
//         assert!(stress_index < 0.3);
//         assert_eq!(get_stress_level(stress_index), "Low");
//     }

//     #[test]
//     fn test_stress_index_high() {
//         let data = SensorData {
//             temperature: 40.0,
//             humidity: 90.0,
//             noise: 90.0,
//             heart_rate: 120.0,
//             motion: true,
//             timestamp: "2026-01-24T00:00:00Z".to_string(),
//         };
//         let stress_index = calculate_stress_index(&data);
//         assert!(stress_index > 0.6);
//         assert_eq!(get_stress_level(stress_index), "High");
//     }
// }

// // ---------------------- Integration test (API) ----------------------
// #[actix_rt::test]
// async fn test_health_endpoint() {
//     let app =
//         test::init_service(App::new().route("/health", actix_web::web::get().to(health))).await;
//     let req = test::TestRequest::get().uri("/health").to_request();
//     let resp = test::call_service(&app, req).await;
//     assert!(resp.status().is_success());
// }

// #[actix_rt::test]
// async fn test_get_realtime_endpoint() {
//     use esms_backend::{redis, AppState};

//     let state = actix_web::web::Data::new(AppState {
//         redis_client: Arc::new(Mutex::new(
//             redis::Client::open("redis://127.0.0.1:6379").unwrap(),
//         )),
//         mysql_pool: mysql_async::Pool::new("mysql://user:pass@localhost/db"),
//         in_memory: Arc::new(Mutex::new(VecDeque::new())),
//     });

//     let app = test::init_service(
//         App::new()
//             .app_data(state.clone())
//             .route("/api/realtime", actix_web::web::get().to(get_realtime)),
//     )
//     .await;

//     let req = test::TestRequest::get().uri("/api/realtime").to_request();
//     let resp = test::call_service(&app, req).await;
//     assert!(resp.status().is_success());
// }

use actix_web::{test, App};
use esms_backend::{
    calculate_stress_index, get_realtime, get_stress_level, health, EnhancedSensorData, SensorData,
};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
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

// ---------------------- Integration tests ----------------------
#[cfg(test)]
mod integration_tests {
    use super::*;
    use esms_backend::{redis, AppState};

    /// Setup AppState connecting to Docker services in CI/CD
    async fn setup_app_state() -> actix_web::web::Data<AppState> {
        // Retry loop for Redis
        let redis_client = loop {
            match redis::Client::open("redis://127.0.0.1:6379") {
                Ok(client) => break client,
                Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        };

        // MySQL pool
        let mysql_pool = mysql_async::Pool::new("mysql://esms_user:esms_pass@127.0.0.1/esms_db");

        actix_web::web::Data::new(AppState {
            redis_client: Arc::new(Mutex::new(redis_client)),
            mysql_pool,
            in_memory: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    /// Health endpoint test (safe to run anywhere)
    #[actix_rt::test]
    async fn test_health_endpoint() {
        let app =
            test::init_service(App::new().route("/health", actix_web::web::get().to(health))).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    /// Realtime API test (requires Redis & MySQL)
    /// Marked #[ignore] so it doesn't fail in local dev without Docker
    #[actix_rt::test]
    #[ignore]
    async fn test_get_realtime_endpoint() {
        let state = setup_app_state().await;

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/api/realtime", actix_web::web::get().to(get_realtime)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/realtime").to_request();
        let resp = test::call_service(&app, req).await;

        // Assert success even if no data exists yet
        assert!(resp.status().is_success());

        // Optional: check if response is JSON array (empty or filled)
        let body_bytes = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body_bytes).unwrap_or("");
        assert!(body_str.starts_with('[') && body_str.ends_with(']'));
    }
}
