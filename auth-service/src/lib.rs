pub mod domain;
/// Modules
pub mod router;
pub mod routes;
pub mod services;
pub mod utils;

use router::app_routes;
/// Imports
use routes::{
        handle_login, handle_login_or_signup, handle_logout, handle_signup, handle_verify_2fa,
        handle_verify_token,
};
use utils::fetch_assets;

use axum::{
        extract::Json,
        http::StatusCode,
        response::IntoResponse,
        routing::{get, get_service, post, MethodRouter},
        Router,
};
use tower_http::services::{ServeDir, ServeFile};

/// Types
pub type AppResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// Application
#[derive(Debug)]
pub struct Application {
        server: axum::serve::Serve<tokio::net::TcpListener, Router, Router>,
        pub address: String,
}

impl Application {
        pub async fn build<S: Into<String>>(address: S) -> AppResult<Self> {
                let asset_dir = fetch_assets();
                let router = app_routes(asset_dir);

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
