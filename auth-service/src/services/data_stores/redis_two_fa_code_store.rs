use std::sync::Arc;

use color_eyre::eyre::Context;
use redis::{Commands, Connection};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::domain::data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError};
use crate::Email;

pub struct RedisTwoFACodeStore {
    conn: Arc<RwLock<Connection>>,
}

impl RedisTwoFACodeStore {
    pub fn new(conn: Arc<RwLock<Connection>>) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl TwoFACodeStore for RedisTwoFACodeStore {
    #[tracing::instrument(name = "RedisTwoFACodeStore::add_code", skip_all)]
    async fn add_code(
        &mut self,
        email: &Email,
        code: TwoFACode,
        login_attempt_id: LoginAttemptId,
    ) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(&email);
        let two_fa = TwoFATuple(
            code.as_ref().to_string(),
            login_attempt_id.as_ref().to_string(),
        );
        let json = serde_json::to_string(&two_fa)
            .wrap_err("Failed to serialize TwoFATuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;

        self.conn
            .write()
            .await
            .set_ex(key, json, TEN_MINUTES_IN_SECONDS)
            .wrap_err("failed to set 2FA code in Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)
    }

    #[tracing::instrument(name = "RedisTwoFACodeStore::remove_code", skip_all)]
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        self.conn
            .write()
            .await
            .del(key)
            .wrap_err("Failed to delete 2FA code from Redis")
            .map_err(TwoFACodeStoreError::UnexpectedError)
    }

    #[tracing::instrument(name = "RedisTwoFACodeStore::get_code", skip_all)]
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(TwoFACode, LoginAttemptId), TwoFACodeStoreError> {
        let key = get_key(email);
        let json = self
            .conn
            .write()
            .await
            .get::<String, String>(key)
            .map_err(|_| TwoFACodeStoreError::LoginAttemptIdNotFound)?;
        let TwoFATuple(two_fa_code, login_attempt_id): TwoFATuple = serde_json::from_str(&json)
            .wrap_err("Failed to deserialize TwoFATuple")
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        let login_attempt_id = LoginAttemptId::parse(&login_attempt_id)
            .map_err(TwoFACodeStoreError::UnexpectedError)?;
        let two_fa_code =
            TwoFACode::parse(two_fa_code).map_err(TwoFACodeStoreError::UnexpectedError)?;
        Ok((two_fa_code, login_attempt_id))
    }
}

// Tuple struct to hold the 2FA code and login attempt ID
#[derive(Serialize, Deserialize)]
struct TwoFATuple(pub String, pub String);

const TEN_MINUTES_IN_SECONDS: u64 = 600;
const TWO_FA_CODE_PREFIX: &str = "two_fa_code:";

fn get_key(email: &Email) -> String {
    format!("{}{}", TWO_FA_CODE_PREFIX, email.as_ref())
}
