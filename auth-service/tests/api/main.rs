mod helpers;
mod login;
mod logout;
mod root;
mod routes;
mod signup;
mod verify_2fa;
mod verify_token;

pub use crate::helpers::{get_random_email, TestApp};
pub use auth_service::routes::{LoginPayload, SignupPayload, Verify2FAPayload, VerifyTokenPayload};

pub type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>;
