use std::sync::Arc;

use color_eyre::eyre::Context;
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
    #[tracing::instrument(name = "add_banned_token", skip_all)]
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let ttl: u64 = TOKEN_TTL_SECONDS
            .try_into()
            .wrap_err("failed to cast TTL to u64")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        let _: () = conn
            .set_ex(key, true, ttl)
            .wrap_err("Failed to add banned token to Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }

    #[tracing::instrument(name = "is_token_banned", skip_all)]
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let result: Option<bool> = conn
            .get(key)
            .wrap_err("Failed to check if token exists in Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(result.unwrap_or(false))
    }

    #[tracing::instrument(name = "remove_banned_token", skip_all)]
    async fn remove_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        let mut conn = self.conn.write().await;
        let key = get_key(token);
        let _: () = conn
            .del(key)
            .wrap_err("Failed to remove token from Redis")
            .map_err(BannedTokenStoreError::UnexpectedError)?;
        Ok(())
    }
}

// We are using a key prefix to prevent collisions and organize data!
const BANNED_TOKEN_KEY_PREFIX: &str = "banned_token:";

fn get_key(token: &str) -> String {
    format!("{}{}", BANNED_TOKEN_KEY_PREFIX, token)
}
