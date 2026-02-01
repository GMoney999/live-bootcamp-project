// src/routes/signup.rs
use crate::{
        domain::{AuthAPIError, Email, Password, User, UserStore},
        AppState, ErrorResponse,
};
use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
        Json as JsonData,
};
use regex::Regex;

/// POST – /signup
pub async fn handle_signup<T>(
        State(state): State<AppState<T>>,
        Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, AuthAPIError>
where
        T: UserStore,
{
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");

        // If the signup route is called with invalid input (ex: an incorrectly formatted email address or password), a 400 HTTP status code should be returned.
        let (req_email, req_pwd) = validate_credentials(&payload.email, &payload.password)?;

        // If one attempts to create a new user with an existing email address, a 409 HTTP status code should be returned.
        // NOTE: Scope created to prevent deadlock. Read lock is dropped before write
        let user_exists = {
                let store_guard = state.user_store.read().await;
                store_guard.get_user(&req_email).await.is_ok()
        }; // NOTE:  Read lock dropped here

        /// If user already exists, return 409
        if user_exists {
                return Err(AuthAPIError::UserAlreadyExists);
        }

        let user = User::new(req_email, req_pwd, payload.requires_2fa);

        // NOTE: Now safe to acquire write lock
        match state.user_store.write().await.add_user(user).await {
                Ok(_) => Ok(SignupResponse::new("User created successfully!")),
                Err(_) => Err(AuthAPIError::UserAlreadyExists),
        }
}

fn validate_credentials(email: &str, password: &str) -> Result<(Email, Password), AuthAPIError> {
        let email = Email::parse(email)?;
        let pwd = Password::parse(password)?;

        Ok((email, pwd))
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignupResponse {
        pub message: String,
}
impl SignupResponse {
        pub fn new(message: impl Into<String>) -> Self {
                let message: String = message.into();
                Self {
                        message,
                }
        }
}

impl IntoResponse for SignupResponse {
        fn into_response(self) -> axum::response::Response {
                (StatusCode::CREATED, Json(self)).into_response()
        }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SignupPayload {
        email: String,
        password: String,
        #[serde(rename = "requires2FA")]
        requires_2fa: bool,
}

impl SignupPayload {
        pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
                Self {
                        email,
                        password,
                        requires_2fa,
                }
        }
        pub fn email(&self) -> &String {
                &self.email
        }
        pub fn password(&self) -> &String {
                &self.password
        }
        pub fn requires_2fa(&self) -> bool {
                self.requires_2fa
        }
        pub fn email_to_owned(&self) -> String {
                self.email.clone()
        }
        pub fn password_to_owned(&self) -> String {
                self.password.clone()
        }
}
