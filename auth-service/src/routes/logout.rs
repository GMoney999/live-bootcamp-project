// src/routes/logout.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_logout() -> impl IntoResponse {
        println!("->> {:<12} – handle_logout", "HANDLER");

        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogoutPayload {
        token: String,
}
