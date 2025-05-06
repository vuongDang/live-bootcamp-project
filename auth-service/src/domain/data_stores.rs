use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::user::User;

/// This module defines the data stores used in the application.
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<&User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

/// This enum defines the possible errors that can occur when interacting with the user store.
#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

/// This module defines the data store for banned tokens.
#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), UserStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, UserStoreError>;
    async fn remove_banned_token(&mut self, token: &str) -> Result<(), UserStoreError>;
}
