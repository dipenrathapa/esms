// src/api.rs
use actix_web::{web, HttpResponse, Result};
use chrono::Utc;

use crate::state::AppState;

pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    })))
}

pub async fn get_realtime(state: web::Data<AppState>) -> Result<HttpResponse> {
    let mem = state.memory.lock().await;
    let data: Vec<_> = mem.iter().rev().take(60).cloned().collect();
    Ok(HttpResponse::Ok().json(data))
}