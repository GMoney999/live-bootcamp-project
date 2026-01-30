use crate::ErrorResponse;
use axum::{http::StatusCode, response::IntoResponse, Json};

pub enum AuthAPIError {
        UserAlreadyExists,
        InvalidCredentials,
        UnexpectedError,
}

impl IntoResponse for AuthAPIError {
        fn into_response(self) -> axum::response::Response {
                let (status, error_message) = match self {
                        AuthAPIError::UserAlreadyExists => {
                                (StatusCode::CONFLICT, "User already exists")
                        }
                        AuthAPIError::InvalidCredentials => {
                                (StatusCode::BAD_REQUEST, "Invalid credentials")
                        }
                        AuthAPIError::UnexpectedError => {
                                (StatusCode::INTERNAL_SERVER_ERROR, "Unexpected error")
                        }
                };
                let body = Json(ErrorResponse {
                        error: error_message.to_string(),
                });
                (status, body).into_response()
        }
}
