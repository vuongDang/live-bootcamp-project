use std::collections::HashMap;

use crate::domain::user::User;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[derive(Default, Debug)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users.get(email).ok_or(UserStoreError::UserNotFound)
    }
    
    /// Returns  Ok(()) if the user is found and the password is correct.
    /// Returns `UserStoreError::UserNotFound` if the user can not be found.
    /// Returns `UserStoreError::InvalidCredentials` if the password is incorrect.
    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        match self.get_user(email) {
            Ok(user) if user.password == password => Ok(()),
            Ok(_) => Err(UserStoreError::InvalidCredentials),
            Err(UserStoreError::UserNotFound) => Err(UserStoreError::UserNotFound),
            Err(_) => unreachable!("Unexpected error while validating user"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("toto@foo.com".to_string(), "password123".to_string(), true);
        assert!(store.add_user(user.clone()).is_ok());
        let res = store.add_user(user.clone());
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::UserAlreadyExists);
        assert!(store.users.keys().len() == 1);
        assert_eq!(store.users.get(&user.email).unwrap(), &user);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("toto@foo.com".to_string(), "password123".to_string(), true);
        assert!(store.add_user(user.clone()).is_ok());
        assert!(store.get_user(&user.email).is_ok());
        assert_eq!(store.get_user(&user.email).unwrap(), &user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new("toto@foo.com".to_string(), "password123".to_string(), true);
        assert!(store.add_user(user.clone()).is_ok());
        assert!(store.validate_user(&user.email, &user.password).is_ok());
        let res = store.validate_user(&user.email, "wrongpassword");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::InvalidCredentials);
        let res = store.validate_user("non_existent_email", "password123");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::UserNotFound);
    }
}