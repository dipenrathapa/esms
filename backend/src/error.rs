// src/error.rs
use actix_web::HttpResponse;

#[derive(thiserror::Error, Debug)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Internal server error")]
    Internal,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Redis error: {0}")]
    Redis(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("TCP connection error: {0}")]
    TcpConnection(String),
}

impl actix_web::ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ApiError::Internal => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "internal_error",
                "message": "An internal error occurred"
            })),
            ApiError::Database(msg) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "database_error",
                    "message": msg
                }))
            }
            ApiError::Redis(msg) => HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "redis_error",
                "message": msg
            })),
            ApiError::Validation(msg) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": "validation_error",
                "message": msg
            })),
            ApiError::TcpConnection(msg) => {
                HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": "tcp_connection_error",
                    "message": msg
                }))
            }
        }
    }
}