use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};

use color_eyre::eyre::eyre;
use color_eyre::eyre::Result;
use sqlx::PgPool;

use crate::domain::data_stores::{UserStore, UserStoreError};
use crate::{Email, Password, User};

pub struct PostgresUserStore {
    pool: PgPool,
}

impl PostgresUserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserStore for PostgresUserStore {
    #[tracing::instrument(name = "Adding user to db", skip_all)]
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        // Check if the user already exists
        let existing_user = sqlx::query!(
            r#"
            SELECT * FROM users WHERE email = $1
            "#,
            user.email.0
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing_user.is_some() {
            return Err(UserStoreError::UserAlreadyExists);
        }

        // Hash the password before storing it
        let password_hash = compute_password_hash(user.password.0)
            .await
            .map_err(|e| UserStoreError::UnexpectedError(e.into()))?;

        // Store the user in the database
        sqlx::query!(
            r#"
            INSERT INTO users (email, password_hash, requires_2fa)
            VALUES ($1, $2, $3)
            "#,
            user.email.0,
            password_hash,
            user.requires_2fa
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument(name = "Getting user from db", skip_all)]
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        // Retrieve the user from the database
        let user = sqlx::query!(
            r#"
            SELECT *
            FROM users
            WHERE email = $1
            "#,
            email.0
        )
        .fetch_optional(&self.pool)
        .await?;

        match user {
            Some(user) => Ok(User {
                email: Email::parse(&user.email)
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                password: Password::parse(&user.password_hash)
                    .map_err(|e| UserStoreError::UnexpectedError(eyre!(e)))?,
                requires_2fa: user.requires_2fa,
            }),
            None => Err(UserStoreError::UserNotFound),
        }
    }

    #[tracing::instrument(name = "Validating user", skip_all)]
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        // Retrieve the user from the database
        let record = sqlx::query!(
            r#"
            SELECT password_hash
            FROM users
            WHERE email = $1
            "#,
            email.0
        )
        .fetch_optional(&self.pool)
        .await?;

        match record {
            Some(record) => {
                // Verify the password hash
                verify_password_hash(record.password_hash, password.0.clone())
                    .await
                    .map_err(|_| UserStoreError::InvalidCredentials)?;
                Ok(())
            }
            None => Err(UserStoreError::UserNotFound),
        }
    }
}

// Helper function to verify if a given password matches an expected hash
#[tracing::instrument(name = "Verifying password hash", skip_all)]
async fn verify_password_hash(
    expected_password_hash: String,
    password_candidate: String,
) -> color_eyre::eyre::Result<()> {
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let expected_password_hash: PasswordHash<'_> =
                PasswordHash::new(&expected_password_hash)?;
            Argon2::default()
                .verify_password(password_candidate.as_bytes(), &expected_password_hash)
                .map_err(|e| e.into())
        })
    })
    .await?
}

// Helper function to hash passwords before persisting them in the database.
#[tracing::instrument(name = "Computing password hash", skip_all)]
async fn compute_password_hash(password: String) -> color_eyre::eyre::Result<String> {
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || {
        current_span.in_scope(|| {
            let salt: SaltString = SaltString::generate(&mut rand::thread_rng());
            let password_hash = Argon2::new(
                Algorithm::Argon2id,
                Version::V0x13,
                Params::new(15000, 2, 1, None)?,
            )
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

            Ok(password_hash)
        })
    })
    .await?
}
