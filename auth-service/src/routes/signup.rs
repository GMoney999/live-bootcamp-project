// src/routes/signup.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

/// POST – /signup
pub async fn handle_signup(payload: Json<SignupPayload>) -> impl IntoResponse {
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");

        StatusCode::OK.into_response()
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
        pub fn into_response(self) -> (StatusCode, String) {
                match self {
                        /// User Created
                        Self::UserCreated => {
                                (StatusCode::CREATED, "User created successfully!".to_owned())
                        }

                        /// Invalid Input
                        Self::InvalidInput {
                                error: e,
                        } => (StatusCode::BAD_REQUEST, e.to_owned()),

                        /// Email Already Exists
                        Self::EmailAlreadyExists {
                                error: e,
                        } => (StatusCode::CONFLICT, e.to_owned()),

                        /// Unprocessable Content
                        Self::UnprocessableContent => {
                                (StatusCode::UNPROCESSABLE_ENTITY, "".to_owned())
                        }

                        /// Unexpected Error
                        Self::UnexpectedError {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, e.to_owned()),
                }
        }
}
