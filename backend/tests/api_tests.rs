use actix_web::{test, App};
use esms_backend::{AppState, get_realtime, health}; // adjust your crate name
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::Client as RedisClient;
use mysql_async::Pool;
use tokio_util::sync::CancellationToken;

#[actix_rt::test]
async fn test_health_endpoint() {
    let app = test::init_service(App::new().route("/health", actix_web::web::get().to(health))).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_rt::test]
async fn test_get_realtime_empty_memory() {
    let state = AppState {
        redis: Arc::new(Mutex::new(RedisClient::open("redis://127.0.0.1/").unwrap())),
        mysql: Pool::new("mysql://user:pass@localhost/test_db").unwrap(),
        memory: Arc::new(Mutex::new(VecDeque::new())),
        config: esms_backend::AppConfig {
            redis_url: "redis://127.0.0.1/".to_string(),
            mysql_url: "mysql://user:pass@localhost/test_db".to_string(),
            bind_addr: "127.0.0.1:0".to_string(),
            use_serial: false,
            serial_tcp_host: "127.0.0.1".to_string(),
            serial_tcp_port: 5555,
        },
        shutdown_token: CancellationToken::new(),
        retry_config: Default::default(),
    };

    let app = test::init_service(
        App::new()
            .app_data(actix_web::web::Data::new(state))
            .route("/api/realtime", actix_web::web::get().to(get_realtime)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/realtime").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
