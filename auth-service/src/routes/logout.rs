// src/routes/logout.rs
use axum::{
        extract::State,
        http::StatusCode,
        response::IntoResponse,
};
use axum_extra::extract::{
        cookie::{Cookie, SameSite},
        CookieJar,
};

use crate::{
        domain::BannedTokenStoreError,
        utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
        AppState, HandlerResult,
};

pub async fn handle_logout(
        state: State<AppState>,
        jar: CookieJar,
) -> (CookieJar, HandlerResult<impl IntoResponse>) {
        println!("->> {:<12} – handle_logout", "HANDLER");
        let token = match jar.get(JWT_COOKIE_NAME) {
                Some(cookie) => cookie.value().to_owned(),
                None => return (jar, Err(LogoutError::MissingToken.into())),
        };

        if token.is_empty() {
                return (jar, Err(LogoutError::InvalidToken.into()));
        }

        if validate_token(&state.banned_token_store, &token).await.is_err() {
                return (jar, Err(LogoutError::InvalidToken.into()));
        }

        if let Err(error) = state.banned_token_store.write().await.ban_token(token).await {
                match error {
                        BannedTokenStoreError::TokenAlreadyBanned => {
                                return (jar, Err(LogoutError::InvalidToken.into()))
                        }
                }
        }

        let removal_cookie = Cookie::build((JWT_COOKIE_NAME, ""))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .build();
        let jar = jar.remove(removal_cookie);

        (jar, Ok(StatusCode::OK))
}

pub enum LogoutError {
        /// 400
        MissingToken,
        /// 401
        InvalidToken,
}
