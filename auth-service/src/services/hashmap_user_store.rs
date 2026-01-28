use std::collections::HashMap;

use crate::domain::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
        UserAlreadyExists,
        UserNotFound,
        InvalidCredentials,
        UnexpectedError,
}

pub struct HashmapUserStore {
        #[cfg_attr(test, allow(dead_code))]
        pub(crate) users: HashMap<String, User>,
}

impl HashmapUserStore {
        pub fn new() -> Self {
                Self {
                        users: HashMap::<String, User>::new(),
                }
        }
}

impl HashmapUserStore {
        #[cfg(test)]
        pub(crate) fn insert_user_unchecked(&mut self, email: String, user: User) {
                self.users.insert(email, user);
        }

        // Test-only helper to directly access the HashMap
        #[cfg(test)]
        pub(crate) fn get_users_ref(&self) -> &HashMap<String, User> {
                &self.users
        }

        pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
                if self.users.contains_key(user.email()) {
                        return Err(UserStoreError::UserAlreadyExists);
                };
                self.users.insert(user.email_to_owned(), user);

                Ok(())
        }

        pub fn get_user(&self, email: &str) -> Result<User, UserStoreError> {
                match self.users.get(email) {
                        Some(user) => Ok(user.clone()),
                        None => Err(UserStoreError::UserNotFound),
                }
        }

        pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
                let user = self.get_user(email)?;
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
                let user = User::new("test@example.com", "password", false);

                let result = store.add_user(user.clone());

                assert!(result.is_ok());
                // Direct HashMap access instead of get_user
                assert_eq!(store.get_users_ref().get("test@example.com").unwrap(), &user);
        }

        #[tokio::test]
        async fn test_get_user() {
                let mut store = HashmapUserStore::new();
                let user = User::new("test@example.com", "password", false);

                // Direct insert instead of add_user
                store.insert_user_unchecked("test@example.com".to_string(), user.clone());

                assert_eq!(store.get_user("test@example.com").unwrap(), user);
        }

        #[tokio::test]
        async fn test_validate_user() {
                let mut store = HashmapUserStore::new();

                let user = User::new("test@example.com", "password", false);

                store.add_user(user.clone()).unwrap();

                assert!(store.validate_user("test@example.com", "password").is_ok());
        }
}
