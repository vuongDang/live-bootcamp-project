use std::sync::Arc;

use redis::{Commands, Connection};
use tokio::sync::RwLock;

use crate::{
    domain::data_stores::{BannedTokenStore, BannedTokenStoreError},
    utils::auth::TOKEN_TTL_SECONDS,
};

pub struct RedisBannedTokenStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisBannedTokenStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let _: () = conn
            .set_ex(key, true, TOKEN_TTL_SECONDS as u64)
            .map_err(|_| {
                BannedTokenStoreError::UnexpectedError("Failed to add banned token".to_string())
            })?;
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let result: Option<bool> = conn.get(key).map_err(|_| {
            BannedTokenStoreError::UnexpectedError("Failed to check banned token".to_string())
        })?;
        Ok(result.unwrap_or(false))
    }

    async fn remove_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let _: () = conn.del(key).map_err(|_| {
            BannedTokenStoreError::UnexpectedError("Failed to remove banned token".to_string())
        })?;
        Ok(())
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
