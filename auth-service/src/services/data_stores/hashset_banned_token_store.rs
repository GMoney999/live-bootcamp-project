// src/services/hashset_banned_token_store.rs
use async_trait::async_trait;

use crate::domain::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub struct HashsetBannedTokenStore {
        banned_tokens: HashSet<String>,
}

impl HashsetBannedTokenStore {
        pub fn new() -> Self {
                Self::default()
        }
}

#[async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
        async fn ban_token(&mut self, token: String) -> Result<(), BannedTokenStoreError> {
                if self.banned_tokens.contains(&token) {
                        Err(BannedTokenStoreError::TokenAlreadyBanned)
                } else {
                        self.banned_tokens.insert(token);
                        Ok(())
                }
        }

        async fn is_banned(&self, token: String) -> bool {
                self.banned_tokens.contains(&token)
        }
}
