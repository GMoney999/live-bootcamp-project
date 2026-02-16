use async_trait::async_trait;

use crate::domain::{login_attempt_id::LoginAttemptId, two_fa_code::TwoFACode, Email, Password};

use super::User;

#[async_trait]
pub trait UserStore: Send + Sync {
        async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
        async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
        async fn validate_user(
                &self,
                email: &Email,
                password: &Password,
        ) -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
        UserAlreadyExists,
        UserNotFound,
        InvalidCredentials,
        UnexpectedError,
}

#[async_trait]
pub trait BannedTokenStore: Send + Sync {
        async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
        async fn is_banned(&self, token: String) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
        TokenAlreadyBanned,
}

#[async_trait]
pub trait TwoFACodeStore: Send + Sync {
        async fn add_code(
                &mut self,
                email: Email,
                login_attempt_id: LoginAttemptId,
                code: TwoFACode,
        ) -> Result<(), TwoFACodeStoreError>;
        async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
        async fn get_code(
                &self,
                email: &Email,
        ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
        CodeNotFound,
        CodeAlreadyExists,
}
