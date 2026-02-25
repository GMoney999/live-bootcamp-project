// src/services//data_stores/postgres_user_store.rs
use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::{
        data_stores::{UserStore, UserStoreError},
        Email, HashedPassword, User,
};

pub struct PostgresUserStore {
        pool: PgPool,
}

impl PostgresUserStore {
        pub fn new(pool: PgPool) -> Self {
                Self {
                        pool,
                }
        }
}

#[async_trait]
impl UserStore for PostgresUserStore {
        async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
                sqlx::query!(
                        r#"
                        INSERT INTO users (email, password_hash, requires_2fa)
                        VALUES ($1, $2, $3)
                        "#,
                        user.email_str(),
                        user.password_str(),
                        user.requires_2fa(),
                )
                .execute(&self.pool)
                .await
                .map_err(|e| match e {
                        sqlx::Error::Database(db_err) if db_err.constraint().is_some() => {
                                UserStoreError::UserAlreadyExists
                        }
                        _ => UserStoreError::UnexpectedError,
                })?;
                Ok(())
        }

        async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
                let row = sqlx::query!(
                        r#"
                        SELECT email, password_hash, requires_2fa
                        FROM users
                        WHERE email = $1
                        "#,
                        email.as_str()
                )
                .fetch_one(&self.pool)
                .await
                .map_err(|e| match e {
                        sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
                        _ => UserStoreError::UnexpectedError,
                })?;

                let email: Email =
                        Email::parse(&row.email).map_err(|_| UserStoreError::UnexpectedError)?;
                let password: HashedPassword =
                        HashedPassword::parse_password_hash(row.password_hash)
                                .map_err(|_| UserStoreError::UnexpectedError)?;
                let user = User::new(email, password, row.requires_2fa);

                Ok(user)
        }

        async fn validate_user(
                &self,
                email: &Email,
                raw_password: &str,
        ) -> Result<(), UserStoreError> {
                let user = self.get_user(email).await?;

                user.password()
                        .verify_raw_password(raw_password)
                        .await
                        .map_err(|_| UserStoreError::InvalidCredentials)?;

                Ok(())
        }
}
