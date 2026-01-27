// src/routes/logout.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_logout() -> impl IntoResponse {
        println!("->> {:<12} – handle_logout", "HANDLER");

        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LogoutPayload {
        token: String,
}

enum LogoutState {
        Success,
        InvalidInput {
                error: String,
        },
        InvalidJWT {
                error: String,
        },
        Unexpected {
                error: String,
        },
}

impl LogoutState {
        pub fn as_response(&self) -> (StatusCode, String) {
                match self {
                        Self::Success => (StatusCode::OK, "".to_owned()),
                        Self::InvalidInput {
                                error: e,
                        } => (StatusCode::BAD_REQUEST, e.to_owned()),
                        Self::InvalidJWT {
                                error: e,
                        } => (StatusCode::UNAUTHORIZED, e.to_owned()),
                        Self::Unexpected {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, e.to_owned()),
                }
        }
}
