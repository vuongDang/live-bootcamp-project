mod domain;
mod routes;
mod services;
pub mod utils;
use crate::domain::data_stores::UserStore;
use crate::routes::{login, logout, signup, verify_2fa, verify_token};
use crate::services::hashmap_user_store::HashmapUserStore;
use axum::http::Method;
use axum::{routing::post, serve::Serve, Router};
use domain::data_stores::BannedTokenStore;
pub use domain::email::Email;
pub use domain::error;
use services::hashset_banned_token_store::HashsetBannedTokenStore;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

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
            .layer(cors);

        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?.to_string();
        let server = axum::serve(listener, router);

        // Create a new Application instance and return it
        Ok(Application { server, address })
    }

    pub async fn run(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

// Using a type alias to improve readability!
pub type UserStoreType = Arc<RwLock<dyn UserStore>>;
pub type BannedTokenStoreType = Arc<RwLock<dyn BannedTokenStore>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
    pub banned_token_store: BannedTokenStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType, banned_token_store: BannedTokenStoreType) -> Self {
        Self {
            user_store,
            banned_token_store,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user_store: Arc::new(RwLock::new(HashmapUserStore::default())),
            banned_token_store: Arc::new(RwLock::new(HashsetBannedTokenStore::default())),
        }
    }
}
