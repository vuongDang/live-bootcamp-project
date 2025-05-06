use std::collections::HashMap;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    email::Email,
};

#[derive(Default)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (TwoFACode, LoginAttemptId)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: &Email,
        code: TwoFACode,
        login_attempt_id: LoginAttemptId,
    ) -> Result<(), TwoFACodeStoreError> {
        self.codes.insert(email.clone(), (code, login_attempt_id));
        Ok(())
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(TwoFACode, LoginAttemptId), TwoFACodeStoreError> {
        if let Some(code) = self.codes.get(email) {
            Ok(code.clone())
        } else {
            Err(TwoFACodeStoreError::LoginAttemptIdNotFound)
        }
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        self.codes.remove(email);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hashmap_two_fa_code_store() {
        let mut store = HashmapTwoFACodeStore::default();
        let email = Email::parse("foo@bar.com").unwrap();
        let code = TwoFACode::default();
        let login_attempt_id = LoginAttemptId::default();
        assert!(store
            .add_code(&email, code.clone(), login_attempt_id.clone())
            .await
            .is_ok());
        let retrieved_code = store.get_code(&email).await;
        assert!(retrieved_code.is_ok());
        assert_eq!(retrieved_code.unwrap(), (code, login_attempt_id));
        assert!(store.remove_code(&email).await.is_ok());
        let retrieved_code = store.get_code(&email).await;
        assert!(retrieved_code.is_err());
    }
}
