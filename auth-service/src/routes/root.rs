// src/routes/root.rs
use axum::{http::StatusCode, response::IntoResponse};

pub async fn handle_login_or_signup() -> impl IntoResponse {
        println!("->> {:<12} – handle_login_or_signup", "HANDLER");
        StatusCode::OK.into_response()
}
