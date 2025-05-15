use crate::domain::data_stores::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;

#[derive(Default, Debug, Clone)]
pub struct HashsetBannedTokenStore {
    banned_tokens: HashSet<String>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        self.banned_tokens.insert(token.to_string());
        Ok(())
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.banned_tokens.contains(token))
    }

    async fn remove_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError> {
        self.banned_tokens.remove(token);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_banned_token() {
        let mut store = HashsetBannedTokenStore::default();
        let token = "token1";
        store.add_banned_token(token).await.unwrap();
        assert!(store.is_token_banned(token).await.unwrap());
        assert!(!store.is_token_banned("token2").await.unwrap());
        store.remove_banned_token(token).await.unwrap();
        assert!(!store.is_token_banned(token).await.unwrap());
    }
}
