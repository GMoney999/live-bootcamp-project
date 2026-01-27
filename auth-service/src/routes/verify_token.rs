// src/routes/verify_token.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_verify_token(Json(payload): Json<VerifyTokenPayload>) -> impl IntoResponse {
        println!("->> {:<12} — handle_verify_token – {payload:?}", "HANDLER");

        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct VerifyTokenPayload {
        token: String,
}

impl VerifyTokenPayload {
        pub fn new(token: String) -> Self {
                Self {
                        token,
                }
        }
}

enum TokenState {
        Valid,
        Invalid {
                error: String,
        },
        Unprocessable,
        Unexpected {
                error: String,
        },
}

impl TokenState {
        pub fn as_response(&self) -> (StatusCode, String) {
                match self {
                        Self::Valid => (StatusCode::OK, "".to_owned()),
                        Self::Unprocessable => (StatusCode::UNPROCESSABLE_ENTITY, "".to_owned()),
                        Self::Invalid {
                                error: e,
                        } => (StatusCode::UNAUTHORIZED, e.to_owned()),
                        Self::Unexpected {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, e.to_owned()),
                }
        }
}
