use crate::{
        domain::{EmailError, PasswordError, TwoFACodeStoreError, UserStoreError},
        routes::{LogoutError, TokenError},
        utils::auth::GenerateTokenError,
};
use axum::{http::StatusCode, response::IntoResponse, Json};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ErrorResponse {
        pub error: String,
}

pub enum AuthAPIError {
        /// 400
        InvalidCredentials,
        /// 400
        MissingToken,
        /// 401
        Unauthorized,
        /// 401
        InvalidToken,
        /// 404
        UserNotFound,
        /// 409
        UserAlreadyExists,
        /// 422
        UnprocessableContent,
        /// 500
        UnexpectedError,
}

impl IntoResponse for AuthAPIError {
        fn into_response(self) -> axum::response::Response {
                let (status, error_message) = match self {
                        /// 400
                        AuthAPIError::InvalidCredentials => {
                                (StatusCode::BAD_REQUEST, "Invalid credentials")
                        }
                        /// 400
                        AuthAPIError::MissingToken => {
                                (StatusCode::BAD_REQUEST, "Missing JWT auth token")
                        }

                        /// 401
                        AuthAPIError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
                        /// 401
                        AuthAPIError::InvalidToken => {
                                (StatusCode::UNAUTHORIZED, "Invalid JWT auth token")
                        }

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

impl From<LogoutError> for AuthAPIError {
        fn from(err: LogoutError) -> Self {
                match err {
                        LogoutError::MissingToken => AuthAPIError::MissingToken,
                        LogoutError::InvalidToken => AuthAPIError::InvalidToken,
                }
        }
}

impl From<TokenError> for AuthAPIError {
        fn from(err: TokenError) -> Self {
                match err {
                        TokenError::InvalidToken => AuthAPIError::InvalidToken,
                        TokenError::MalformedInput => AuthAPIError::UnprocessableContent,
                }
        }
}

impl From<GenerateTokenError> for AuthAPIError {
        fn from(err: GenerateTokenError) -> Self {
                AuthAPIError::UnexpectedError
        }
}

impl From<TwoFACodeStoreError> for AuthAPIError {
        fn from(err: TwoFACodeStoreError) -> Self {
                match err {
                        TwoFACodeStoreError::CodeNotFound => AuthAPIError::Unauthorized,
                        TwoFACodeStoreError::CodeAlreadyExists => AuthAPIError::UserAlreadyExists,
                }
        }
}
