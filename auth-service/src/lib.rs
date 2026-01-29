pub mod domain;
/// Modules
pub mod router;
pub mod routes;
pub mod services;
pub mod utils;

use axum::{
        extract::Json,
        http::StatusCode,
        response::IntoResponse,
        routing::{get, get_service, post, MethodRouter},
        Router,
};
use router::app_routes;
use routes::{
        handle_login, handle_login_or_signup, handle_logout, handle_signup, handle_verify_2fa,
        handle_verify_token,
};
use services::hashmap_user_store::HashmapUserStore;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use utils::fetch_assets;

/// Types
pub type AppResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;
pub type UserStoreType = Arc<RwLock<HashmapUserStore>>;

#[derive(Clone)]
pub struct AppState {
        pub user_store: UserStoreType,
}
impl AppState {
        pub fn new(user_store: UserStoreType) -> Self {
                Self {
                        user_store,
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
        pub async fn build<S: Into<String>>(app_state: AppState, address: S) -> AppResult<Self> {
                let asset_dir = fetch_assets();
                let router = app_routes(app_state, asset_dir);

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
