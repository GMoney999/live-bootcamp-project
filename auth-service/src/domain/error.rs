use crate::{
        domain::{EmailError, PasswordError, UserStoreError},
        ErrorResponse,
};
use axum::{http::StatusCode, response::IntoResponse, Json};

pub enum AuthAPIError {
        UserAlreadyExists,
        UserNotFound,
        Unauthorized,
        InvalidCredentials,
        UnprocessableContent,
        UnexpectedError,
}

impl IntoResponse for AuthAPIError {
        fn into_response(self) -> axum::response::Response {
                let (status, error_message) = match self {
                        /// 400
                        AuthAPIError::InvalidCredentials => {
                                (StatusCode::BAD_REQUEST, "Invalid credentials")
                        }
                        /// 401
                        AuthAPIError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
                        /// 404
                        AuthAPIError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
                        /// 409
                        AuthAPIError::UserAlreadyExists => {
                                (StatusCode::CONFLICT, "User already exists")
                        }
                        /// 422
                        AuthAPIError::UnprocessableContent => {
                                (StatusCode::UNPROCESSABLE_ENTITY, "Unprocessable content")
                        }
                        /// 500
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

impl From<UserStoreError> for AuthAPIError {
        fn from(err: UserStoreError) -> Self {
                match err {
                        UserStoreError::UserNotFound => AuthAPIError::UserNotFound,
                        UserStoreError::InvalidCredentials => AuthAPIError::InvalidCredentials,
                        UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
                        UserStoreError::UnexpectedError => AuthAPIError::UnexpectedError,
                }
        }
}

impl From<EmailError> for AuthAPIError {
        fn from(err: EmailError) -> Self {
                AuthAPIError::InvalidCredentials
        }
}

impl From<PasswordError> for AuthAPIError {
        fn from(err: PasswordError) -> Self {
                AuthAPIError::InvalidCredentials
        }
}
