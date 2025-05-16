use color_eyre::eyre::eyre;
use color_eyre::eyre::Report;
use color_eyre::eyre::Result;
use rand::Rng;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::user::User;

/// This module defines the data stores used in the application.
#[async_trait::async_trait]
pub trait UserStore: Send + Sync {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

/// This enum defines the possible errors that can occur when interacting with the user store.
#[derive(Debug, Error)]
pub enum UserStoreError {
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

impl PartialEq for UserStoreError {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::UserAlreadyExists, Self::UserAlreadyExists)
                | (Self::UserNotFound, Self::UserNotFound)
                | (Self::InvalidCredentials, Self::InvalidCredentials)
                | (Self::UnexpectedError(_), Self::UnexpectedError(_))
        )
    }
}

impl From<sqlx::Error> for UserStoreError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => UserStoreError::UserNotFound,
            sqlx::Error::Database(e) => UserStoreError::UnexpectedError(e.into()),
            _ => UserStoreError::UnexpectedError(err.into()),
        }
    }
}

/// This module defines the data store for banned tokens.
#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
    async fn remove_banned_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
}

#[derive(Debug, Error)]
pub enum BannedTokenStoreError {
    #[error("Connection error")]
    ConnectionError,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[async_trait::async_trait]
pub trait TwoFACodeStore: Send + Sync {
    async fn add_code(
        &mut self,
        email: &Email,
        code: TwoFACode,
        login_attempt_id: LoginAttemptId,
    ) -> Result<(), TwoFACodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(TwoFACode, LoginAttemptId), TwoFACodeStoreError>;
    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;
}

#[derive(Debug, Error)]
pub enum TwoFACodeStoreError {
    #[error("Login attempt id not found")]
    LoginAttemptIdNotFound,
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);
impl LoginAttemptId {
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        LoginAttemptId(id)
    }

    pub fn parse(id: &str) -> Result<Self> {
        if Uuid::parse_str(id).is_ok() {
            Ok(LoginAttemptId(id.to_string()))
        } else {
            Err(eyre!("Invalid LoginAttemptId: {}", id))
        }
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        LoginAttemptId::new()
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TwoFACode(String);
impl TwoFACode {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let random: u32 = rng.gen_range(0..1_000_000);
        let random_string = format!("{:06}", random);
        TwoFACode(random_string)
    }

    pub fn parse(code: String) -> Result<Self> {
        // Ensure `code` is a valid 6-digit code
        let is_valid = code.len() == 6 && code.chars().all(|c| c.is_digit(10));
        is_valid
            .then(|| TwoFACode(code.clone()))
            .ok_or_else(|| eyre!("Invalid TwoFACode: {}", code))
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        TwoFACode::new()
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
