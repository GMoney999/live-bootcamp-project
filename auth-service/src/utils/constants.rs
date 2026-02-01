// src/utils/constants.rs
use dotenvy::dotenv;
use lazy_static::lazy_static;

// Define a lazily evaluated static. lazy_static is needed because std_env::var is not a const function.
lazy_static! {
        pub static ref JWT_SECRET: String = set_token();
}

/// This value determines how long the JWT auth token is valid for
pub const TOKEN_TTL_SECONDS: i64 = 600; // 10 minutes

pub fn set_token() -> String {
        dotenv().ok();
        let secret = std::env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set");

        if secret.is_empty() {
                panic!("JWT Secret cannot be empty");
        }

        secret
}

pub mod env {
        pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
}

pub const JWT_COOKIE_NAME: &str = "jwt";
