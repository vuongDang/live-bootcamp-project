use crate::domain::data_stores::BannedTokenStore;
use crate::domain::data_stores::TwoFACodeStore;
use crate::domain::data_stores::UserStore;
use crate::domain::EmailClient;
use crate::get_postgres_pool;
use crate::services::data_stores::hashmap_two_fa_code_store::HashmapTwoFACodeStore;
use crate::services::data_stores::hashmap_user_store::HashmapUserStore;
use crate::services::data_stores::hashset_banned_token_store::HashsetBannedTokenStore;
use crate::services::data_stores::redis_two_fa_code_store::RedisTwoFACodeStore;
use crate::services::email_clients::postmark_email_client::PostmarkEmailClient;
use crate::utils::constants::prod;
use crate::utils::constants::DATABASE_URL;
use crate::utils::constants::POSTMARK_AUTH_TOKEN;
use crate::utils::constants::REDIS_HOST_NAME;
use crate::Email;
use crate::PostgresUserStore;
use crate::RedisBannedTokenStore;
use reqwest::Client;
use sqlx::PgPool;
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

    /// Creates a new AppState with a PostgreSQL user store and a Redis banned token store / two fa code store.
    pub async fn new_ps_redis() -> Self {
        let pg_pool = configure_postgresql().await;
        let user_store = Arc::new(RwLock::new(PostgresUserStore::new(pg_pool)));
        let banned_token_store = Arc::new(RwLock::new(RedisBannedTokenStore::new(Arc::new(
            RwLock::new(configure_redis()),
        ))));
        let two_fa_code_store = Arc::new(RwLock::new(RedisTwoFACodeStore::new(Arc::new(
            RwLock::new(configure_redis()),
        ))));
        let email_client = Arc::new(RwLock::new(configure_postmark_email_client()));

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
                crate::services::email_clients::mock_email_client::MockEmailClient::default(),
            )),
        }
    }
}

async fn configure_postgresql() -> PgPool {
    // Create a new database connection pool
    let pg_pool = get_postgres_pool(&DATABASE_URL)
        .await
        .expect("Failed to create Postgres connection pool!");

    // Run database migrations against our test database!
    sqlx::migrate!()
        .run(&pg_pool)
        .await
        .expect("Failed to run migrations");

    pg_pool
}

fn configure_redis() -> redis::Connection {
    crate::get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}

fn configure_postmark_email_client() -> PostmarkEmailClient {
    let http_client = Client::builder()
        .timeout(prod::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(
        prod::email_client::BASE_URL.to_owned(),
        Email::parse(&prod::email_client::SENDER.to_owned()).unwrap(),
        POSTMARK_AUTH_TOKEN.to_owned(),
        http_client,
    )
}
