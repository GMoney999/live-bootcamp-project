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
use sqlx::{postgres::PgPoolOptions, Executor, PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::{
        cors::CorsLayer,
        services::{ServeDir, ServeFile},
};
use utils::fetch_assets;
use uuid::Uuid;

use crate::{
        domain::{two_fa_code, BannedTokenStore, EmailClient, TwoFACodeStore, UserStore},
        services::data_stores::{
                postgres_user_store::PostgresUserStore, HashmapTwoFACodeStore,
                HashsetBannedTokenStore, MockEmailClient,
        },
        utils::constants::{
                env::{DROPLET_URL_ENV_VAR, LOCALHOST_URL_ENV_VAR},
                get_env_var, DATABASE_URL,
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

#[derive(Default, Clone)]
pub struct AppStateBuilder {
        pub user_store: Option<UserStoreType>,
        pub banned_token_store: Option<BannedTokenStoreType>,
        pub two_fa_code_store: Option<TwoFACodeStoreType>,
        pub email_client: Option<EmailClientType>,
}

impl AppStateBuilder {
        pub fn new() -> Self {
                AppStateBuilder::default()
        }

        pub fn user_store(mut self, user_store: UserStoreType) -> Self {
                self.user_store = Some(user_store);
                self
        }

        pub fn banned_token_store(
                mut self,
                banned_token_store: BannedTokenStoreType,
        ) -> Self {
                self.banned_token_store = Some(banned_token_store);
                self
        }

        pub fn two_fa_code_store(mut self, two_fa_code_store: TwoFACodeStoreType) -> Self {
                self.two_fa_code_store = Some(two_fa_code_store);
                self
        }

        pub fn email_client(mut self, email_client: EmailClientType) -> Self {
                self.email_client = Some(email_client);
                self
        }

        pub fn build(self) -> AppState {
                AppState {
                        user_store: self.user_store.expect("User Store"),
                        banned_token_store: self
                                .banned_token_store
                                .expect("Banned Token Store"),
                        two_fa_code_store: self.two_fa_code_store.expect("2FA Code Store"),
                        email_client: self.email_client.expect("Email Client"),
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

async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
        // Create a new PostgreSQL connection pool
        PgPoolOptions::new().max_connections(5).connect(url).await
}

/// Production: connect to the existing database and run migrations.
pub async fn init_postgres_pool() -> PgPool {
        let url = DATABASE_URL.to_owned();
        let pool = get_postgres_pool(&url).await.expect("Failed to connect to Postgres");
        sqlx::migrate!().run(&pool).await.expect("Failed to run database migrations");
        pool
}

/// Test-only: create a fresh UUID-named database, run migrations, and return a pool.
/// This gives each test run an isolated, clean database.
pub async fn configure_postgresql() -> PgPool {
        let postgresql_conn_url = DATABASE_URL.to_owned();
        let db_name = Uuid::new_v4().to_string();

        configure_database(&postgresql_conn_url, &db_name).await;

        let postgres_conn_url_with_db_name = format!("{}/{}", postgresql_conn_url, db_name);
        get_postgres_pool(&postgres_conn_url_with_db_name)
                .await
                .expect("Failed to create Postgres connection pool")
}

pub async fn configure_database(db_conn_string: &str, db_name: &str) {
        let connection = PgPoolOptions::new()
                .connect(db_conn_string)
                .await
                .expect("Failed to create Postgres connection pool.");

        connection
                .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
                .await
                .expect("Failed to create database.");

        let db_conn_string = format!("{}/{}", db_conn_string, db_name);

        let connection = PgPoolOptions::new()
                .connect(&db_conn_string)
                .await
                .expect("Failed to create Postgres conenction pool.");

        sqlx::migrate!().run(&connection).await.expect("Failed to migrate the database.");
}

pub fn get_user_store(pool: Pool<Postgres>) -> Arc<RwLock<Box<dyn UserStore + Send + Sync>>> {
        Arc::new(RwLock::new(Box::new(PostgresUserStore::new(pool))))
}

pub fn get_banned_token_store() -> Arc<RwLock<Box<dyn BannedTokenStore + Send + Sync>>> {
        Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::new())))
}

pub fn get_two_fa_code_store() -> Arc<RwLock<Box<dyn TwoFACodeStore + Send + Sync>>> {
        Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::new())))
}

pub fn get_email_client() -> Arc<dyn EmailClient + Send + Sync> {
        Arc::new(MockEmailClient)
}
