use async_trait::async_trait;
use redis::{Connection, TypedCommands};
use tokio::sync::Mutex;

use crate::domain::{Email, LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};

pub struct RedisTwoFACodeStore {
        conn: Mutex<Connection>,
}

impl RedisTwoFACodeStore {
        pub fn new(conn: Connection) -> Self {
                Self {
                        conn: Mutex::new(conn),
                }
        }
}

#[async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
        async fn add_code(
                &mut self,
                email: Email,
                login_attempt_id: LoginAttemptId,
                code: TwoFACode,
        ) -> Result<(), TwoFACodeStoreError> {
                // 1. Create a new key using the get_key helper function.
                let key = get_key(&email);

                // 2. Create a TwoFATuple instance.
                let tuple =
                        TwoFATuple(login_attempt_id.as_ref().to_owned(), code.as_ref().to_owned());

                // 3. Use serde_json::to_string to serialize the TwoFATuple instance into a JSON string.
                let value = serde_json::to_string(&tuple)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                // 4. Call the set_ex command on the Redis connection
                self.conn
                        .lock()
                        .await
                        .set_ex(key, value, TEN_MINUTES_IN_SECONDS)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                Ok(())
        }

        async fn get_code(
                &self,
                email: &Email,
        ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
                // 1. Create a new key using the get_key helper function.
                let key = get_key(email);

                // 2. Call the get command on the Redis connection to get the value stored for the key.
                let value: Option<String> = self
                        .conn
                        .lock()
                        .await
                        .get(key)
                        .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;

                // Handle the case where the key doesn't exist
                let json_string = value.ok_or(TwoFACodeStoreError::LoginAttemptIdNotFound)?;

                // Parse the JSON string into a TwoFATuple
                let tuple: TwoFATuple = serde_json::from_str(&json_string)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                // Parse the login attempt ID string and 2FA code string into proper types
                let login_attempt_id = LoginAttemptId::parse(tuple.0)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;
                let two_fa_code = TwoFACode::parse(tuple.1)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                Ok((login_attempt_id, two_fa_code))
        }

        async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
                let key = get_key(email);
                self.conn
                        .lock()
                        .await
                        .del(key)
                        .map_err(|_| TwoFACodeStoreError::UnexpectedError)?;

                Ok(())
        }
}

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

#[derive(serde::Serialize, serde::Deserialize)]
struct TwoFATuple(pub String, pub String);

fn get_key(email: &Email) -> String {
        format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
