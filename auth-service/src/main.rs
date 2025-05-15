use std::sync::Arc;

use auth_service::utils::constants::{prod, DATABASE_URL};
use auth_service::{get_postgres_pool, Application};
use sqlx::PgPool;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let mut state = auth_service::app_state::AppState::default();
    let pg_pool = configure_postgresql().await;
    let user_store = auth_service::PostgresUserStore::new(pg_pool);
    state.user_store = Arc::new(RwLock::new(user_store));

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
