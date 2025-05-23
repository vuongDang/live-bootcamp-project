use secrecy::Secret;

use super::data_stores::UserStoreError;
use super::email::Email;
use super::password::Password;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub(crate) email: Email,
    pub(crate) password: Password,
    pub(crate) requires_2fa: bool,
}

impl User {
    pub fn new(
        email: String,
        password: Secret<String>,
        requires_2fa: bool,
    ) -> Result<Self, UserStoreError> {
        let email = Email::parse(&email).map_err(|_| UserStoreError::InvalidCredentials)?;
        let password = Password::parse(password).map_err(|_| UserStoreError::InvalidCredentials)?;
        Ok(User {
            email,
            password,
            requires_2fa,
        })
    }

    pub(crate) fn new_with_fake_password(
        email: String,
        password: Secret<String>,
        requires_2fa: bool,
    ) -> Result<Self, UserStoreError> {
        let email = Email::parse(&email).map_err(|_| UserStoreError::InvalidCredentials)?;
        let password = Password::fake(password);
        Ok(User {
            email,
            password,
            requires_2fa,
        })
    }
}
