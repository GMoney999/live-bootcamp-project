use crate::utils::constants::env::{
        DATABASE_URL_ENV_VAR, DROPLET_URL_ENV_VAR, LOCALHOST_URL_ENV_VAR,
};

// src/utils/constants.rs
use super::constants::env::JWT_SECRET_ENV_VAR;
use dotenvy::dotenv;
use lazy_static::lazy_static;

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
lazy_static! {
        pub static ref JWT_SECRET: String = get_env_var(JWT_SECRET_ENV_VAR);
        pub static ref LOCALHOST_URL: String = get_env_var(LOCALHOST_URL_ENV_VAR);
        pub static ref DROPLET_URL: String = get_env_var(DROPLET_URL_ENV_VAR);
        pub static ref DATABASE_URL: String = get_env_var(DATABASE_URL_ENV_VAR);
}

pub mod prod {
        pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
        pub const APP_ADDRESS: &str = "127.0.0.1:3000";
}

pub mod env {
        pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
        pub const LOCALHOST_URL_ENV_VAR: &str = "LOCALHOST_URL";
        pub const DROPLET_URL_ENV_VAR: &str = "DROPLET_URL";
        pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
}

pub fn get_env_var<S: Into<String>>(var: S) -> String {
        dotenv().ok();
        let env_var: String = var.into();
        let secret = std::env::var(&env_var)
                .unwrap_or_else(|_| panic!("{} must be set", env_var.clone()));

        if secret.is_empty() {
                panic!("{} cannot be empty", env_var)
        }

        secret
}

pub const JWT_COOKIE_NAME: &str = "jwt";

/// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes
