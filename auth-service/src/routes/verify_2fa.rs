// src/routes/verify_2fa.rs
use axum::{extract::Json, http::StatusCode, response::IntoResponse};

pub async fn handle_verify_2fa(Json(payload): Json<Verify2FAPayload>) -> impl IntoResponse {
        println!("->> {:<12} — handle_signup – {payload:?}", "HANDLER");
        StatusCode::OK.into_response()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Verify2FAPayload {
        email: String,
        #[serde(rename = "loginAttemptId")]
        login_attempt_id: String,
        code: String,
}

impl Verify2FAPayload {
        pub fn new(email: String, login_attempt_id: String, code: String) -> Self {
                Self {
                        email,
                        login_attempt_id,
                        code,
                }
        }
}

enum Verify2FAState {
        Verified,
        InvalidInput {
                error: String,
        },
        AuthFailed {
                error: String,
        },
        Unprocessable,
        Unexpected {
                error: String,
        },
}

impl Verify2FAState {
        pub fn as_response(&self) -> (StatusCode, String) {
                match self {
                        Self::Verified => (StatusCode::OK, "".to_owned()),
                        Self::Unprocessable => (StatusCode::UNPROCESSABLE_ENTITY, "".to_owned()),
                        Self::InvalidInput {
                                error: e,
                        } => (StatusCode::BAD_REQUEST, e.to_owned()),
                        Self::AuthFailed {
                                error: e,
                        } => (StatusCode::UNAUTHORIZED, e.to_owned()),
                        Self::Unexpected {
                                error: e,
                        } => (StatusCode::INTERNAL_SERVER_ERROR, e.to_owned()),
                }
        }
}

impl IntoResponse for Verify2FAState {
        fn into_response(self) -> axum::response::Response {
                self.as_response().into_response()
        }
}
