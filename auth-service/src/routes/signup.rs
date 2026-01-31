// src/routes/signup.rs
use crate::{
        domain::{AuthAPIError, User, UserStore},
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
/// A 500 HTTP status code should be returned if an unexpected error occurs.
pub async fn handle_signup<T>(
        State(state): State<AppState<T>>,
        Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, AuthAPIError>
where
        T: UserStore,
{
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");

        let req_email = payload.email_to_owned();
        let req_pwd = payload.password_to_owned();

        // If the signup route is called with invalid input (ex: an incorrectly formatted email address), a 400 HTTP status code should be returned.
        if !is_valid_email(&req_email) || !is_valid_pwd(&req_pwd) {
                return Err(AuthAPIError::InvalidCredentials);
        }

        // If one attempts to create a new user with an existing email address, a 409 HTTP status code should be returned.
        // Check with read lock first
        let user_exists = {
                let store_guard = state.user_store.read().await;
                store_guard.get_user(&req_email).await.is_ok()
        }; // Read lock dropped here

        if user_exists {
                return Err(AuthAPIError::UserAlreadyExists);
        }

        let user = User::new(payload.email, payload.password, payload.requires_2fa);

        // Now safe to acquire write lock
        match state.user_store.write().await.add_user(user).await {
                Ok(_) => Ok(SignupResponse::new("User created successfully!")),
                Err(_) => Err(AuthAPIError::UserAlreadyExists),
        }
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

// DO NO
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

fn is_valid_email(email: &str) -> bool {
        let re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email)
}

fn is_valid_pwd(password: &str) -> bool {
        let chars = password.chars().collect::<Vec<char>>();
        chars.len() >= 8
}
