use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

#[derive(Default, Debug)]
pub struct HashmapTwoFACodeStore {
        codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

impl HashmapTwoFACodeStore {
        pub fn new() -> Self {
                Self::default()
        }
}

#[async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
        async fn add_code(
                &mut self,
                email: Email,
                login_attempt_id: LoginAttemptId,
                code: TwoFACode,
        ) -> Result<(), TwoFACodeStoreError> {
                if self.codes.contains_key(&email) {
                        return Err(TwoFACodeStoreError::CodeAlreadyExists);
                }
                self.codes.insert(email, (login_attempt_id, code));
                Ok(())
        }

        async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
                if self.codes.remove(email).is_none() {
                        return Err(TwoFACodeStoreError::CodeNotFound);
                }

                Ok(())
        }

        async fn get_code(
                &self,
                email: &Email,
        ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
                match self.codes.get(email) {
                        Some(login_id_and_code) => Ok(login_id_and_code.clone()),
                        None => Err(TwoFACodeStoreError::CodeNotFound),
                }
        }
}

#[cfg(test)]
mod tests {
        use std::sync::Arc;

        use tokio::sync::Mutex;

        use super::*;

        // Helper function to create test data
        fn create_test_email() -> Email {
                Email::parse("test@example.com").unwrap()
        }

        fn create_test_login_attempt_id() -> LoginAttemptId {
                LoginAttemptId::default()
        }

        fn create_test_2fa_code() -> TwoFACode {
                TwoFACode::parse("123456".to_string()).unwrap()
        }

        #[tokio::test]
        async fn test_add_code_success() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id = create_test_login_attempt_id();
                let code = create_test_2fa_code();

                let result = store.add_code(email.clone(), login_id.clone(), code.clone()).await;

                assert!(result.is_ok());

                // Verify the code was actually stored
                let stored = store.get_code(&email).await.unwrap();
                assert_eq!(stored.0, login_id);
                assert_eq!(stored.1, code);
        }

        #[tokio::test]
        async fn test_add_code_rejects_duplicate() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id1 = create_test_login_attempt_id();
                let code1 = create_test_2fa_code();
                let login_id2 = create_test_login_attempt_id();
                let code2 = TwoFACode::parse("654321".to_string()).unwrap();

                // Add first code - should succeed
                store.add_code(email.clone(), login_id1.clone(), code1.clone()).await.unwrap();

                // Try to add second code - should fail with error
                let result = store.add_code(email.clone(), login_id2, code2).await;
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), TwoFACodeStoreError::CodeAlreadyExists);

                // Verify the first code is still intact (not overwritten)
                let stored = store.get_code(&email).await.unwrap();
                assert_eq!(stored.0, login_id1);
                assert_eq!(stored.1, code1);
        }

        #[tokio::test]
        async fn test_add_code_after_removal() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id1 = create_test_login_attempt_id();
                let code1 = create_test_2fa_code();

                // Add and then remove code
                store.add_code(email.clone(), login_id1, code1).await.unwrap();
                store.remove_code(&email).await.unwrap();

                // Now adding a new code should succeed
                let login_id2 = create_test_login_attempt_id();
                let code2 = TwoFACode::parse("654321".to_string()).unwrap();
                store.add_code(email.clone(), login_id2.clone(), code2.clone()).await.unwrap();

                let stored = store.get_code(&email).await.unwrap();
                assert_eq!(stored.0, login_id2);
                assert_eq!(stored.1, code2);
        }

        #[tokio::test]
        async fn test_get_code_success() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id = create_test_login_attempt_id();
                let code = create_test_2fa_code();

                store.add_code(email.clone(), login_id.clone(), code.clone()).await.unwrap();

                let result = store.get_code(&email).await;

                assert!(result.is_ok());
                let (retrieved_login_id, retrieved_code) = result.unwrap();
                assert_eq!(retrieved_login_id, login_id);
                assert_eq!(retrieved_code, code);
        }

        #[tokio::test]
        async fn test_get_code_email_not_found() {
                let store = HashmapTwoFACodeStore::default();
                let email = create_test_email();

                let result = store.get_code(&email).await;

                assert!(result.is_err());
                assert!(matches!(result.unwrap_err(), TwoFACodeStoreError::CodeNotFound));
        }

        #[tokio::test]
        async fn test_remove_code_success() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id = create_test_login_attempt_id();
                let code = create_test_2fa_code();

                // Add code first
                store.add_code(email.clone(), login_id, code).await.unwrap();

                // Verify it exists
                assert!(store.get_code(&email).await.is_ok());

                // Remove it
                let result = store.remove_code(&email).await;

                assert!(result.is_ok());

                // Verify it's gone
                let get_result = store.get_code(&email).await;
                assert!(get_result.is_err());
                assert!(matches!(get_result.unwrap_err(), TwoFACodeStoreError::CodeNotFound));
        }

        #[tokio::test]
        async fn test_remove_code_email_not_found() {
                let mut store = HashmapTwoFACodeStore::default();
                let email = create_test_email();

                let result = store.remove_code(&email).await;

                assert!(result.is_err());
                assert!(matches!(result.unwrap_err(), TwoFACodeStoreError::CodeNotFound));
        }

        #[tokio::test]
        async fn test_multiple_emails() {
                let mut store = HashmapTwoFACodeStore::default();
                let email1 = Email::parse("user1@example.com").unwrap();
                let email2 = Email::parse("user2@example.com").unwrap();
                let login_id1 = create_test_login_attempt_id();
                let login_id2 = create_test_login_attempt_id();
                let code1 = TwoFACode::parse("111111".to_string()).unwrap();
                let code2 = TwoFACode::parse("222222".to_string()).unwrap();

                // Add codes for both emails
                store.add_code(email1.clone(), login_id1.clone(), code1.clone()).await.unwrap();
                store.add_code(email2.clone(), login_id2.clone(), code2.clone()).await.unwrap();

                // Verify both exist and are correct
                let result1 = store.get_code(&email1).await.unwrap();
                let result2 = store.get_code(&email2).await.unwrap();

                assert_eq!(result1.0, login_id1);
                assert_eq!(result1.1, code1);
                assert_eq!(result2.0, login_id2);
                assert_eq!(result2.1, code2);

                // Remove one and verify the other still exists
                store.remove_code(&email1).await.unwrap();

                assert!(store.get_code(&email1).await.is_err());
                assert!(store.get_code(&email2).await.is_ok());
        }

        #[tokio::test]
        async fn test_default_implementation() {
                let store = HashmapTwoFACodeStore::default();
                let email = create_test_email();

                // Default store should be empty
                let result = store.get_code(&email).await;
                assert!(result.is_err());
                assert!(matches!(result.unwrap_err(), TwoFACodeStoreError::CodeNotFound));
        }

        #[tokio::test]
        async fn test_debug_implementation() {
                let store = HashmapTwoFACodeStore::default();
                let debug_str = format!("{:?}", store);

                assert!(debug_str.contains("HashmapTwoFACodeStore"));
                assert!(debug_str.contains("codes"));
        }

        #[tokio::test]
        async fn test_store_isolation() {
                // Test that different store instances don't interfere with each other
                let mut store1 = HashmapTwoFACodeStore::default();
                let mut store2 = HashmapTwoFACodeStore::default();
                let email = create_test_email();
                let login_id = create_test_login_attempt_id();
                let code = create_test_2fa_code();

                // Add to store1
                store1.add_code(email.clone(), login_id, code).await.unwrap();

                // Verify store1 has it, store2 doesn't
                assert!(store1.get_code(&email).await.is_ok());
                assert!(store2.get_code(&email).await.is_err());
        }

        #[tokio::test]
        async fn test_large_number_of_entries() {
                let mut store = HashmapTwoFACodeStore::default();
                let num_entries = 1000;

                // Add many entries
                for i in 0..num_entries {
                        let email =
                                Email::parse(format!("user{}@example.com", i).as_str()).unwrap();
                        let login_id = create_test_login_attempt_id();
                        let code = TwoFACode::parse(format!("{:06}", i % 1000000)).unwrap();

                        store.add_code(email, login_id, code).await.unwrap();
                }

                // Verify a few random entries exist
                for i in [0, 100, 500, 999] {
                        let email =
                                Email::parse(format!("user{}@example.com", i).as_str()).unwrap();
                        let result = store.get_code(&email).await;
                        assert!(result.is_ok(), "Entry {} should exist", i);
                }

                // Remove half the entries
                for i in 0..num_entries / 2 {
                        let email =
                                Email::parse(format!("user{}@example.com", i).as_str()).unwrap();
                        store.remove_code(&email).await.unwrap();
                }

                // Verify removed entries are gone and remaining entries still exist
                for i in 0..num_entries / 2 {
                        let email =
                                Email::parse(format!("user{}@example.com", i).as_str()).unwrap();
                        assert!(
                                store.get_code(&email).await.is_err(),
                                "Entry {} should be removed",
                                i
                        );
                }

                for i in num_entries / 2..num_entries {
                        let email =
                                Email::parse(format!("user{}@example.com", i).as_str()).unwrap();
                        assert!(
                                store.get_code(&email).await.is_ok(),
                                "Entry {} should still exist",
                                i
                        );
                }
        }

        #[tokio::test]
        async fn test_concurrent_operations() {
                use tokio::task;

                let store = Arc::new(Mutex::new(HashmapTwoFACodeStore::default()));
                let email = create_test_email();
                let login_id = create_test_login_attempt_id();
                let code = create_test_2fa_code();

                // Add initial code
                {
                        let mut store_guard = store.lock().await;
                        store_guard.add_code(email.clone(), login_id, code).await.unwrap();
                }

                // Test that multiple concurrent reads work
                let handles: Vec<_> = (0..10)
                        .map(|_| {
                                let email_clone = email.clone();
                                let store_clone = Arc::clone(&store);
                                tokio::task::spawn(async move {
                                        let store_guard = store_clone.lock().await;
                                        store_guard.get_code(&email_clone).await
                                })
                        })
                        .collect();

                // All reads should succeed
                for handle in handles {
                        let result = handle.await.unwrap();
                        assert!(result.is_ok());
                }
        }
}
