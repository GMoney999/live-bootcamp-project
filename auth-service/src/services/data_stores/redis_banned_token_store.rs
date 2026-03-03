
use async_trait::async_trait;
use redis::{Commands, Connection};
use tokio::sync::Mutex;

use crate::{
        domain::{BannedTokenStore, BannedTokenStoreError},
        utils::constants::TOKEN_TTL_SECONDS,
};

type RedisConnection = Mutex<Connection>;

pub struct RedisBannedTokenStore {
        conn: RedisConnection,
}

impl RedisBannedTokenStore {
        pub fn new(conn: Connection) -> Self {
                Self {
                        conn: Mutex::new(conn),
                }
        }
}

#[async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
        async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
                let key = get_key(&token);
                let ttl = TOKEN_TTL_SECONDS as u64;

                self.conn
                        .lock()
                        .await
                        .set_ex::<_, _, ()>(key, true, ttl)
                        .map_err(|_| BannedTokenStoreError::UnexpectedError)?;

                Ok(())
        }

        async fn is_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
                // Check if the token exists by calling the exists method on the Redis connection
                self.conn
                        .lock()
                        .await
                        .exists::<_, bool>(token)
                        .map_err(|_| BannedTokenStoreError::TokenAlreadyBanned)
        }
}

const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
        format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
