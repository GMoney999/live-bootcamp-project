// src/routes/login.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_login(Json(creds): Json<LoginPayload>) -> impl IntoResponse {
        println!("->> {:<12} – handle_login – {creds:?}", "HANDLER");
        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LoginPayload {
        email: String,
        password: String,
}

impl LoginPayload {
        pub fn new(email: String, password: String) -> Self {
                Self {
                        email,
                        password,
                }
        }
}
