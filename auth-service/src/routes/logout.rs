// src/routes/logout.rs
use axum::{
        extract::{Json, State},
        http::StatusCode,
        response::IntoResponse,
};
use axum_extra::extract::CookieJar;

use crate::{
        domain::{AuthAPIError, UserStore},
        utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
        AppState, HandlerResult,
};

pub async fn handle_logout(jar: CookieJar) -> (CookieJar, HandlerResult<impl IntoResponse>) {
        println!("->> {:<12} – handle_logout", "HANDLER");

        /// Retrieve JWT Token
        let token = match jar.get(JWT_COOKIE_NAME) {
                Some(cookie) => cookie.value().to_owned(),
                None => return (jar, Err(LogoutError::MissingToken.into())),
        };

        /// Make sure JWT isn't empty
        if token.is_empty() {
                return (jar, Err(LogoutError::InvalidToken.into()));
        };

        /// Validate JWT
        if (validate_token(&token).await).is_err() {
                return (jar, Err(LogoutError::InvalidToken.into()));
        }

        /// Remove JWT from cookie jar
        let jar = jar.remove(JWT_COOKIE_NAME);

        (jar, Ok(StatusCode::OK))
}

pub enum LogoutError {
        /// 400
        MissingToken,
        /// 401
        InvalidToken,
}
