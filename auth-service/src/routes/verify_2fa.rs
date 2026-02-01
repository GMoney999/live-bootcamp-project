// src/routes/verify_2fa.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_verify_2fa(Json(payload): Json<Verify2FAPayload>) -> impl IntoResponse {
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");
        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Verify2FAPayload {
        email: String,
        #[serde(rename = "loginAttemptId")]
        login_attempt_id: String,
        code: String,
}
