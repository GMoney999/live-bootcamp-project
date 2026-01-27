// src/routes/login.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_login(Json(creds): Json<LoginPayload>) -> impl IntoResponse {
        println!("->> {:<12} – handle_login – {creds:?}", "HANDLER");
        StatusCode::OK.into_response()
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

enum LoginState {
        Success,
        Requires2FA,
        InvalidInput {
                error: String,
        },
        AuthFail {
                error: String,
        },
        Unprocessable,
        Unexpected {
                error: String,
        },
}

impl LoginState {
        pub fn as_response(&self) -> (StatusCode, String) {
                match self {
                        Self::Success => (StatusCode::OK, "".to_owned()),
                        Self::Unprocessable => (StatusCode::UNPROCESSABLE_ENTITY, "".to_owned()),
                        Self::Requires2FA => (StatusCode::PARTIAL_CONTENT, "".to_owned()),
                        Self::InvalidInput {
                                error: e,
                        } => (StatusCode::BAD_REQUEST, e.to_owned()),
                        Self::AuthFail {
                                error: e,
                        } => (StatusCode::UNAUTHORIZED, e.to_owned()),
                        Self::Unexpected {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, e.to_owned()),
                }
        }
}
