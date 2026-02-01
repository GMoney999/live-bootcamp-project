use crate::domain::{Email, Password, User, UserStore, UserStoreError};
use std::collections::HashMap;

#[derive(Default)]
pub struct HashmapUserStore {
        #[cfg_attr(test, allow(dead_code))]
        pub(crate) users: HashMap<Email, User>,
}

impl HashmapUserStore {
        pub fn new() -> Self {
                Self::default()
        }

        #[cfg(test)]
        pub(crate) fn insert_user_unchecked(&mut self, email: Email, user: User) {
                self.users.insert(email, user);
        }

        #[cfg(test)]
        pub(crate) fn get_users_ref(&self) -> &HashMap<Email, User> {
                &self.users
        }
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
        async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
                if self.users.contains_key(user.email()) {
                        return Err(UserStoreError::UserAlreadyExists);
                };
                self.users.insert(user.email_to_owned(), user);

                Ok(())
        }

        async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
                match self.users.get(email) {
                        Some(user) => Ok(user.clone()),
                        None => Err(UserStoreError::UserNotFound),
                }
        }

        async fn validate_user(
                &self,
                email: &Email,
                password: &Password,
        ) -> Result<(), UserStoreError> {
                let user = self.get_user(email).await?;
                if user.password() != password {
                        return Err(UserStoreError::InvalidCredentials);
                }

                Ok(())
        }
}

#[cfg(test)]
mod tests {
        use super::*;

        #[tokio::test]
        async fn test_add_user() {
                let mut store = HashmapUserStore::new();
                let email = Email::parse("test@example.com").unwrap();
                let password = Password::parse("ValidPassword123").unwrap();

                let user = User::new(email.clone(), password, false);

                let result = store.add_user(user.clone()).await;

                assert!(result.is_ok());
                // Direct HashMap access instead of get_user
                assert_eq!(store.get_users_ref().get(&email).unwrap(), &user);
        }

        #[tokio::test]
        async fn test_get_user() {
                let mut store = HashmapUserStore::new();
                let email = Email::parse("test@example.com").unwrap();
                let password = Password::parse("ValidPassword123").unwrap();

                let user = User::new(email.clone(), password, false);

                // Direct insert instead of add_user
                store.insert_user_unchecked(email.clone(), user.clone());

                assert_eq!(store.get_user(&email).await.unwrap(), user);
        }

        #[tokio::test]
        async fn test_validate_user() {
                let mut store = HashmapUserStore::new();
                let email = Email::parse("test@example.com").unwrap();
                let password = Password::parse("ValidPassword123").unwrap();

                let user = User::new(email.clone(), password.clone(), false);

                store.add_user(user.clone()).await.unwrap();

                assert!(store.validate_user(&email, &password).await.is_ok());
        }
}
