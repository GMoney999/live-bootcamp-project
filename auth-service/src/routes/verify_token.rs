// src/routes/verify_token.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_verify_token(Json(payload): Json<VerifyTokenPayload>) -> impl IntoResponse {
        println!("->> {:<12} — handle_verify_token – {payload:?}", "HANDLER");

        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerifyTokenPayload {
        token: String,
}

impl VerifyTokenPayload {
        pub fn new(token: String) -> Self {
                Self {
                        token,
                }
        }
}
