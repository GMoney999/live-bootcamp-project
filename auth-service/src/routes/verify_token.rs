// src/routes/verify_token.rs
use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
};

use crate::{domain::AuthAPIError, utils::auth::validate_token, AppState, HandlerResult};

// If the JSON object is missing or malformed, a 422 HTTP status code will be sent back (handled by Axum's JSON extractor)
pub async fn handle_verify_token(
        State(state): State<AppState>,
        Json(payload): Json<VerifyTokenPayload>,
) -> HandlerResult<impl IntoResponse> {
        println!("->> {:<12} — handle_verify_token – REDACTED", "HANDLER");

        if payload.token.is_empty() {
                return Err(TokenError::MalformedInput.into());
        }

        // Validate the token
        validate_token(&state.banned_token_store, &payload.token)
                .await
                .map_err(|_| TokenError::InvalidToken)?;

        Ok(StatusCode::OK.into_response())
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

#[derive(Debug)]
pub enum TokenError {
        /// 401
        InvalidToken,
        /// 422
        MalformedInput,
}
