mod domain;
mod routes;
mod services;
pub use domain::error;

use axum::{routing::post, serve::Serve, Router};
use std::error::Error;
use tower_http::services::ServeDir;
use crate::routes::{login, logout, signup, verify_2fa, verify_token};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::data_stores::UserStore;
use crate::services::hashmap_user_store::HashmapUserStore;

// This struct encapsulates our application-related logic.
pub struct Application {
    server: Serve<Router, Router>,
    // address is exposed as a public field
    // so we have access to it in tests.
    pub address: String,
}

impl Application {
    pub async fn build(app_state: AppState, address: &str) -> Result<Self, Box<dyn Error>> {
        let router = Router::new()
            .nest_service("/", ServeDir::new("assets"))
            .route("/signup", post(signup))
            .route("/login", post(login))
            .route("/logout", post(logout))
            .route("/verify-2fa", post(verify_2fa))
            .route("/verify-token", post(verify_token))
            .with_state(app_state);

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

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStoreType,
}

impl AppState {
    pub fn new(user_store: UserStoreType) -> Self {
        Self { user_store }
    }
}


impl Default for AppState {
    fn default() -> Self {
        Self {
            user_store: Arc::new(RwLock::new(HashmapUserStore::default())),
        }
    }
}