use std::sync::Arc;

use auth_service::utils::constants::{prod, DATABASE_URL, REDIS_HOST_NAME};
use auth_service::{get_postgres_pool, Application};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let mut state = auth_service::app_state::AppState::default();

    // Use PostgreSQL for user store
    let pg_pool = configure_postgresql().await;
    let user_store = auth_service::PostgresUserStore::new(pg_pool);
    state.user_store = Arc::new(RwLock::new(user_store));

    // Use Redis for banned token store
    let redis_banned_token_store =
        auth_service::RedisBannedTokenStore::new(Arc::new(RwLock::new(configure_redis())));
    state.banned_token_store = Arc::new(RwLock::new(redis_banned_token_store));

    let app = Application::build(state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build the app");
    app.run().await.expect("Failed to run the app");
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
    auth_service::get_redis_client(REDIS_HOST_NAME.to_owned())
        .expect("Failed to get Redis client")
        .get_connection()
        .expect("Failed to get Redis connection")
}
