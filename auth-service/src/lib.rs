// src/lib.rs
// Modules
pub mod domain;
pub mod router;
pub mod routes;
pub mod services;
pub mod utils;

// Imports
use axum::{
        extract::Json,
        http::{HeaderValue, Method, StatusCode},
        response::IntoResponse,
        routing::{get, get_service, post, MethodRouter},
        Router,
};
use domain::AuthAPIError;
use reqwest::Url;
use router::app_routes;
use routes::{
        handle_login, handle_login_or_signup, handle_logout, handle_signup, handle_verify_2fa,
        handle_verify_token,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{
        cors::CorsLayer,
        services::{ServeDir, ServeFile},
};
use utils::fetch_assets;

use crate::{
        domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore},
        utils::constants::{
                env::{DROPLET_URL_ENV_VAR, LOCALHOST_URL_ENV_VAR},
                get_env_var,
        },
};

/// Types
pub type AppResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;
pub type UserStoreType = Arc<RwLock<Box<dyn UserStore + Send + Sync>>>;
pub type BannedTokenStoreType = Arc<RwLock<Box<dyn BannedTokenStore + Send + Sync>>>;
pub type TwoFACodeStoreType = Arc<RwLock<Box<dyn TwoFACodeStore + Send + Sync>>>;
pub type EmailClientType = Arc<dyn EmailClient + Send + Sync>;
pub type HandlerResult<T> = core::result::Result<T, AuthAPIError>;

pub struct AppState {
        pub user_store: UserStoreType,
        pub banned_token_store: BannedTokenStoreType,
        pub two_fa_code_store: TwoFACodeStoreType,
        pub email_client: EmailClientType,
}

impl AppState {
        pub fn new(
                user_store: UserStoreType,
                banned_token_store: BannedTokenStoreType,
                two_fa_code_store: TwoFACodeStoreType,
                email_client: EmailClientType,
        ) -> Self {
                Self {
                        user_store,
                        banned_token_store,
                        two_fa_code_store,
                        email_client,
                }
        }
}

impl Clone for AppState {
        fn clone(&self) -> Self {
                Self {
                        user_store: Arc::clone(&self.user_store),
                        banned_token_store: Arc::clone(&self.banned_token_store),
                        two_fa_code_store: Arc::clone(&self.two_fa_code_store),
                        email_client: Arc::clone(&self.email_client),
                }
        }
}

/// Application
#[derive(Debug)]
pub struct Application {
        server: axum::serve::Serve<tokio::net::TcpListener, Router, Router>,
        pub address: String,
}

impl Application {
        pub async fn build(app_state: AppState, address: impl Into<String>) -> AppResult<Self> {
                let asset_dir = fetch_assets();

                let allowed_origins = get_allowed_origins()?;
                let cors = get_cors(allowed_origins);

                let router = app_routes(app_state, cors, asset_dir);

                let addr: String = address.into();
                let listener = tokio::net::TcpListener::bind(&addr).await?;
                let address = listener.local_addr()?.to_string();

                let server = axum::serve(listener, router);

                Ok(Application {
                        server,
                        address,
                })
        }

        pub async fn run(self) -> Result<(), std::io::Error> {
                println!("Spinning up application...");
                println!("Running on {}", self.address);
                self.server.await
        }
}

fn get_allowed_origins() -> Result<[HeaderValue; 2], Box<dyn std::error::Error>> {
        let localhost_url_header = get_env_var(LOCALHOST_URL_ENV_VAR).parse::<HeaderValue>()?;
        let droplet_url_header = get_env_var(DROPLET_URL_ENV_VAR).parse::<HeaderValue>()?;

        Ok([localhost_url_header, droplet_url_header])
}

fn get_cors(origins: [HeaderValue; 2]) -> CorsLayer {
        CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_credentials(true)
                .allow_origin(origins)
}
