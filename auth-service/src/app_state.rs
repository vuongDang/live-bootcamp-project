use crate::domain::data_stores::BannedTokenStore;
use crate::domain::data_stores::TwoFACodeStore;
use crate::domain::data_stores::UserStore;
use crate::domain::EmailClient;
use crate::services::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use crate::services::hashmap_user_store::HashmapUserStore;
use crate::services::hashset_banned_token_store::HashsetBannedTokenStore;
use std::sync::Arc;
use tokio::sync::RwLock;

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStore>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore>>;
pub type TwoFACodeStoreType = Arc<RwLock<dyn TwoFACodeStore>>;
pub type EmailClientType = Arc<RwLock<dyn EmailClient>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_client: EmailClientType,
}

impl AppState {
    pub fn new(
        user_store: UserStoreType,
        banned_token_store: BannedTokenStoreType,
        two_fa_code_store: TwoFACodeStoreType,
        email_client: EmailClientType,
    ) -> Self {
        Self {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user_store: Arc::new(RwLock::new(HashmapUserStore::default())),
            banned_token_store: Arc::new(RwLock::new(HashsetBannedTokenStore::default())),
            two_fa_code_store: Arc::new(RwLock::new(HashmapTwoFACodeStore::default())),
            email_client: Arc::new(RwLock::new(
                crate::services::mock_email_client::MockEmailClient::default(),
            )),
        }
    }
}
