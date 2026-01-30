// src/routes/signup.rs
use crate::{
        domain::{AuthAPIError, User},
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
pub async fn handle_signup(
        State(state): State<AppState>,
        Json(payload): Json<SignupPayload>,
) -> Result<impl IntoResponse, AuthAPIError> {
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");

        let req_email = payload.email_to_owned();
        let req_pwd = payload.password_to_owned();

        // If the signup route is called with invalid input (ex: an incorrectly formatted email address), a 400 HTTP status code should be returned.
        if !is_valid_email(&req_email) || !is_valid_pwd(&req_pwd) {
                return Err(AuthAPIError::InvalidCredentials);
        }

        // If one attempts to create a new user with an existing email address, a 409 HTTP status code should be returned.
        if state.user_store.read().await.get_user(&req_email).is_ok() {
                return Err(AuthAPIError::UserAlreadyExists);
        }

        let user = User::new(payload.email, payload.password, payload.requires_2fa);

        let mut user_store = state.user_store.write().await;

        match user_store.add_user(user) {
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

// DO NOT MODIFY
#[derive(Debug)]
enum SignupState {
        UserCreated,
        InvalidInput {
                error: String,
        },
        EmailAlreadyExists {
                error: String,
        },
        UnprocessableContent,
        UnexpectedError {
                error: String,
        },
}
impl SignupState {
        pub fn into_response(self) -> (StatusCode, Json<SignupResponse>) {
                match self {
                        /// User Created
                        Self::UserCreated => (
                                StatusCode::CREATED,
                                Json(SignupResponse::new("User created successfully!")),
                        ),

                        /// Invalid Input
                        Self::InvalidInput {
                                error: e,
                        } => (StatusCode::BAD_REQUEST, Json(SignupResponse::new(e))),

                        /// Email Already Exists
                        Self::EmailAlreadyExists {
                                error: e,
                        } => (StatusCode::CONFLICT, Json(SignupResponse::new(e))),

                        /// Unprocessable Content
                        Self::UnprocessableContent => (
                                StatusCode::UNPROCESSABLE_ENTITY,
                                Json(SignupResponse::new("Unprocessable content.")),
                        ),

                        /// Unexpected Error
                        Self::UnexpectedError {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, Json(SignupResponse::new(e))),
                }
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
