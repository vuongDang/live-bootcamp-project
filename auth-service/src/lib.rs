pub mod app_state;
mod domain;
pub mod routes;
mod services;
pub mod utils;
use crate::routes::{login, logout, signup, verify_2fa, verify_token};
pub use crate::services::email_clients;
use app_state::AppState;
use axum::http::Method;
use axum::{routing::post, serve::Serve, Router};
pub use domain::data_stores::{LoginAttemptId, TwoFACode};
pub use domain::error;
pub use domain::{email::Email, password::Password, user::User};
use redis::{Client, RedisResult};
pub use services::data_stores::postgres_user_store::PostgresUserStore;
pub use services::data_stores::redis_banned_token_store::RedisBannedTokenStore;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::error::Error;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use utils::tracing::*;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        // Allow the app service(running on our local machine and in production) to call the auth service
        let allowed_origins = [
            "http://localhost:8000".parse()?,
            "http://[157.245.129.227]:8000".parse()?,
        ];

        let cors = CorsLayer::new()
            .allow_methods([Method::POST, Method::GET])
            .allow_origin(allowed_origins)
            // Allow cookies to be included in requests
            .allow_credentials(true);

        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state)
            .layer(cors)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(make_span_with_request_id)
                    .on_request(on_request)
                    .on_response(on_response),
            );

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        tracing::info!("Starting server on {}", self.address);
        self.server.await
    }
}

pub async fn get_postgres_pool(url: &str) -> Result<PgPool, sqlx::Error> {
    // Create a new PostgreSQL connection pool
    PgPoolOptions::new().max_connections(5).connect(url).await
}

pub fn get_redis_client(redis_hostname: String) -> RedisResult<Client> {
    let redis_url = format!("redis://{}/", redis_hostname);
    redis::Client::open(redis_url)
}
