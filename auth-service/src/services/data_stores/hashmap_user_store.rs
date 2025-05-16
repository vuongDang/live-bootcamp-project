use crate::domain::data_stores::UserStore;
use crate::domain::data_stores::UserStoreError;
use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::user::User;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct HashmapUserStore {
    users: HashMap<Email, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        if self.users.contains_key(&user.email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            self.users.insert(user.email.clone(), user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError> {
        self.users
            .get(email)
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }

    /// Returns  Ok(()) if the user is found and the password is correct.
    /// Returns `UserStoreError::UserNotFound` if the user can not be found.
    /// Returns `UserStoreError::InvalidCredentials` if the password is incorrect.
    async fn validate_user(
        &self,
        email: &Email,
        password: &Password,
    ) -> Result<(), UserStoreError> {
        match self.get_user(email).await {
            Ok(user) if &user.password == password => Ok(()),
            Ok(_) => Err(UserStoreError::InvalidCredentials),
            Err(UserStoreError::UserNotFound) => Err(UserStoreError::UserNotFound),
            Err(_) => unreachable!("Unexpected error while validating user"),
        }
    }
}

#[cfg(test)]
mod tests {
    use secrecy::Secret;

    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "toto@foo.com".to_string(),
            Secret::new("password123".to_string()),
            true,
        )
        .unwrap();
        assert!(store.add_user(user.clone()).await.is_ok());
        let res = store.add_user(user.clone()).await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::UserAlreadyExists);
        assert!(store.users.keys().len() == 1);
        assert_eq!(store.users.get(&user.email).unwrap(), &user);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "toto@foo.com".to_string(),
            Secret::new("password123".to_string()),
            true,
        )
        .unwrap();
        assert!(store.add_user(user.clone()).await.is_ok());
        assert!(store.get_user(&user.email).await.is_ok());
        assert_eq!(store.get_user(&user.email).await.unwrap(), user);
    }

    #[tokio::test]
    async fn test_validate_user() {
        let mut store = HashmapUserStore::default();
        let user = User::new(
            "toto@foo.com".to_string(),
            Secret::new("password123".to_string()),
            true,
        )
        .unwrap();
        assert!(store.add_user(user.clone()).await.is_ok());
        assert!(store
            .validate_user(&user.email, &user.password)
            .await
            .is_ok());
        let res = store
            .validate_user(
                &user.email,
                &Password::parse(Secret::new("wrongpassword123".to_string())).unwrap(),
            )
            .await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::InvalidCredentials);
        let res = store
            .validate_user(
                &Email("non_existent_email".to_string()),
                &Password::parse(Secret::new("password123".to_string())).unwrap(),
            )
            .await;
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), UserStoreError::UserNotFound);
    }
}
