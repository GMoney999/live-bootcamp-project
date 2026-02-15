// src/routes/login.rs
use axum::{
        extract::{FromRequestParts, Json, State},
        http::StatusCode,
        response::IntoResponse,
        Json as JsonData,
};
use axum_extra::extract::CookieJar;

use crate::{
        domain::{AuthAPIError, Email, Password, UserStore},
        utils::auth::generate_auth_cookie,
        AppState, HandlerResult,
};

// If the JSON object is missing or malformed, a 422 HTTP status code will  be sent back (handled by Axum's JSON extractor)
// TODO: AFTER USER LOGS IN WITHOUT 2FA, ISSUE A JWT TOKEN COOKIE
pub async fn handle_login(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<LoginPayload>,
) -> (CookieJar, HandlerResult<impl IntoResponse>) {
        println!("->> {:<12} – handle_login – {payload:?}", "HANDLER");

        // If the JSON object contains invalid credentials (format), a 400 HTTP status code should be sent back.
        let email = match Email::parse(&payload.email) {
                Ok(email) => email,
                Err(e) => return (jar, Err(e.into())),
        };
        let password = match Password::parse(&payload.password) {
                Ok(password) => password,
                Err(e) => return (jar, Err(e.into())),
        };

        let store = state.user_store.read().await;

        // Validate user credentials - return 401 for any validation failure
        if (store.validate_user(&email, &password).await).is_err() {
                return (jar, Err(AuthAPIError::Unauthorized));
        }

        // Generate auth cookie
        let auth_cookie = match generate_auth_cookie(&email) {
                Ok(cookie) => cookie,
                Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
        };

        let updated_jar = jar.add(auth_cookie);

        (updated_jar, Ok(StatusCode::OK.into_response()))
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
