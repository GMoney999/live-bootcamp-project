// src/routes/login.rs
use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
        domain::{
                AuthAPIError, Email, HashedPassword, LoginAttemptId, TwoFACode,
                TwoFACodeStoreError, UserStore,
        },
        utils::auth::generate_auth_cookie,
        AppState, HandlerResult,
};

// If the JSON object is missing or malformed, a 422 HTTP status code will  be sent back (handled by Axum's JSON extractor)
pub async fn handle_login(
        State(state): State<AppState>,
        jar: CookieJar,
        Json(payload): Json<LoginPayload>,
) -> (CookieJar, HandlerResult<impl IntoResponse>) {
        println!("->> {:<12} – handle_login", "HANDLER");

        // If the JSON object contains invalid credentials (format), a 400 HTTP status code should be sent back.
        let email = match Email::parse(&payload.email) {
                Ok(email) => email,
                Err(e) => return (jar, Err(e.into())),
        };
        let raw_password = payload.password;
        let password = match HashedPassword::parse(&raw_password).await {
                Ok(password) => password,
                Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        };

        let store = state.user_store.read().await;

        // Validate user credentials - return 401 for any validation failure
        if (store.validate_user(&email, &raw_password).await).is_err() {
                return (jar, Err(AuthAPIError::Unauthorized));
        }

        // Get User
        let user = match store.get_user(&email).await {
                Ok(user) => user,
                Err(_) => return (jar, Err(AuthAPIError::InvalidCredentials)),
        };

        match user.requires_2fa() {
                true => handle_2fa(user.email(), &state, jar).await,
                false => handle_no_2fa(user.email(), jar).await,
        }
}

#[derive(Debug, Serialize, Deserialize)]
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

async fn handle_2fa(
        email: &Email,
        state: &AppState,
        jar: CookieJar,
) -> (CookieJar, Result<(StatusCode, Json<LoginResponse>), AuthAPIError>) {
        /// Generate a new random login attempt ID and 2FA code
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        /// Store the ID and code in our 2FA code store
        let add_code_result = state
                .two_fa_code_store
                .write()
                .await
                .add_code(email.to_owned(), login_attempt_id.clone(), two_fa_code.clone())
                .await;
        if (add_code_result).is_err() {
                return (jar, Err(TwoFACodeStoreError::CodeAlreadyExists.into()));
        }

        /// Send 2FA Code via Email Client
        let send_email_result = state
                .email_client
                .send_email(email, "2FA: Verify Email", two_fa_code.as_ref())
                .await;
        if (send_email_result).is_err() {
                return (jar, Err(AuthAPIError::UnexpectedError));
        }

        /// Return the login attempt ID to the client
        let response = Json(LoginResponse::TwoFactorAuth(TwoFactorAuthResponse {
                message: "2FA required".to_owned(),
                login_attempt_id: login_attempt_id.as_ref().to_string(),
        }));

        (jar, Ok((StatusCode::PARTIAL_CONTENT, response)))
}

async fn handle_no_2fa(
        email: &Email,
        jar: CookieJar,
) -> (CookieJar, Result<(StatusCode, Json<LoginResponse>), AuthAPIError>) {
        // Generate auth cookie only when 2FA is not required.
        let auth_cookie = match generate_auth_cookie(email) {
                Ok(cookie) => cookie,
                Err(_) => return (jar, Err(AuthAPIError::UnexpectedError)),
        };

        let jar = jar.add(auth_cookie);

        (jar, Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))))
}

// The login route can return 2 possible success responses.
// This enum models each response!
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
        RegularAuth,
        TwoFactorAuth(TwoFactorAuthResponse),
}

impl IntoResponse for LoginResponse {
        fn into_response(self) -> axum::response::Response {
                match self {
                        LoginResponse::RegularAuth => StatusCode::OK.into_response(),
                        LoginResponse::TwoFactorAuth(res) => {
                                (StatusCode::PARTIAL_CONTENT, Json(res)).into_response()
                        }
                }
        }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorAuthResponse {
        pub message: String,
        #[serde(rename = "loginAttemptId")]
        pub login_attempt_id: String,
}
