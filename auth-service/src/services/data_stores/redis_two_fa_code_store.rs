use std::sync::Arc;

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
        let json = serde_json::to_string(&two_fa).map_err(|_| {
            TwoFACodeStoreError::UnexpectedError("Failed to serialize TwoFATuple".to_string())
        })?;
        self.conn
            .write()
            .await
            .set_ex(key, json, TEN_MINUTES_IN_SECONDS)
            .map_err(|_| {
                TwoFACodeStoreError::UnexpectedError("Failed to set 2FA code in Redis".to_string())
            })
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        let key = get_key(email);
        self.conn.write().await.del(key).map_err(|_| {
            TwoFACodeStoreError::UnexpectedError("Failed to delete 2FA code from Redis".to_string())
        })
    }

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
            .map_err(|_| {
                TwoFACodeStoreError::UnexpectedError(
                    "Failed to get 2FA code from Redis".to_string(),
                )
            })?;
        let TwoFATuple(two_fa_code, login_attempt_id): TwoFATuple = serde_json::from_str(&json)
            .map_err(|_| {
                TwoFACodeStoreError::UnexpectedError("Failed to deserialize TwoFATuple".to_string())
            })?;
        let login_attempt_id = LoginAttemptId::parse(&login_attempt_id).map_err(|_| {
            TwoFACodeStoreError::UnexpectedError("Failed to parse login attempt ID".to_string())
        })?;
        let two_fa_code = TwoFACode::parse(two_fa_code).map_err(|_| {
            TwoFACodeStoreError::UnexpectedError("Failed to parse 2FA code".to_string())
        })?;
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
