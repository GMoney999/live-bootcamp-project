// src/routes/login.rs
use axum::{
        extract::{FromRequestParts, Json, State},
        http::StatusCode,
        response::IntoResponse,
        Json as JsonData,
};

use crate::{
        domain::{AuthAPIError, Email, Password, UserStore},
        AppState, HandlerResult,
};

// If the JSON object is missing or malformed, a 422 HTTP status code should be sent back (handled by Axum's JSON extractor)
pub async fn handle_login<T>(
        State(state): State<AppState<T>>,
        Json(payload): Json<LoginPayload>,
) -> HandlerResult<impl IntoResponse>
where
        T: UserStore,
{
        println!("->> {:<12} – handle_login – {payload:?}", "HANDLER");

        // If the JSON object contains invalid credentials (format), a 400 HTTP status code should be sent back.
        let email = Email::parse(&payload.email)?;
        let password = Password::parse(&payload.password)?;

        let store = state.user_store.read().await;

        // Check if user exists - return 401 if not found or password doesn't match
        let user = store.get_user(&email).await?;

        // If the JSON object contains credentials that are valid but incorrect, a 401 HTTP status code should be returned.
        if user.password_str() != password.as_str() {
                return Err(AuthAPIError::Unauthorized);
        }

        Ok((StatusCode::OK, "Login successful!"))
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
