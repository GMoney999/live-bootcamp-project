// src/routes/verify_2fa.rs
use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
};
use axum_extra::extract::CookieJar;

use crate::{
        domain::{
                AuthAPIError, Email, EmailError, HashedPassword, LoginAttemptId, TwoFACode,
                TwoFACodeStoreError,
        },
        utils::auth::{generate_auth_cookie, GenerateTokenError},
        AppState, HandlerResult,
};

// If the request is processed successfully, a 200 HTTP status code should be returned and the JWT auth cookie should be set.
pub async fn handle_verify_2fa(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<Verify2FAPayload>,
) -> (CookieJar, HandlerResult<impl IntoResponse>) {
        println!("->> {:<12} — handle_verify_2fa – {}", "HANDLER", payload.email);

        /// Returns 400 – invalid input
        let (email, login_attempt_id, code) = match verify_payload(payload) {
                Ok(valid_payload) => valid_payload,
                Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        };

        /// Returns 401 – Email not found
        let (store_login_attempt_id, store_code) =
                match state.two_fa_code_store.read().await.get_code(&email).await {
                        Ok(login_attempt_and_id) => login_attempt_and_id,
                        Err(_) => return (jar, Err(TwoFACodeStoreError::CodeNotFound.into())),
                };

        /// Returns 401 – Incorrect login attempt id or 2FA code
        if login_attempt_id.as_ref() != store_login_attempt_id.as_ref()
                || code.as_ref() != store_code.as_ref()
        {
                return (jar, Err(AuthAPIError::Unauthorized));
        }

        /// If credentials match, remove 2FA code from store & set JWT auth-token cookie
        {
                state.two_fa_code_store
                        .write()
                        .await
                        .remove_code(&email)
                        .await
                        .expect("Infalliable");
        }

        /// Returns 500 – Internal error creating auth token
        let cookie = match generate_auth_cookie(&email) {
                Ok(cookie) => cookie,
                Err(_) => return (jar, Err(GenerateTokenError::UnexpectedError.into())),
        };

        let jar = jar.add(cookie);

        (jar, Ok(StatusCode::OK))
}

// Returns 400 if any invalid input
fn verify_payload(
        payload: Verify2FAPayload,
) -> Result<(Email, LoginAttemptId, TwoFACode), AuthAPIError> {
        /// Returns 400 – invalid email
        let req_email = match Email::parse(&payload.email) {
                Ok(email) => email,
                Err(e) => return Err(AuthAPIError::InvalidCredentials),
        };

        let req_login_attempt_id = match LoginAttemptId::parse(payload.login_attempt_id.clone()) {
                Ok(id) => id,
                Err(e) => {
                        eprintln!("{}", e);
                        return Err(AuthAPIError::InvalidCredentials);
                }
        };

        let req_code = match TwoFACode::parse(payload.code.clone()) {
                Ok(code) => code,
                Err(e) => {
                        eprintln!("{}", e);
                        return Err(AuthAPIError::InvalidCredentials);
                }
        };

        Ok((req_email, req_login_attempt_id, req_code))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Verify2FAPayload {
        email: String,
        #[serde(rename = "loginAttemptId")]
        login_attempt_id: String,
        code: String,
}
