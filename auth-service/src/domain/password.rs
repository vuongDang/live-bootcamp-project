use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl Password {
    pub fn parse(password: Secret<String>) -> Result<Self> {
        if Password::is_valid(password.expose_secret()) {
            Ok(Password(password))
        } else {
            Err(eyre!("Password not valid"))
        }
    }

    // Password must be at least 8 characters long and contain at least one digit
    pub fn is_valid(password: &str) -> bool {
        password.len() >= 8 && password.chars().any(|c| c.is_digit(10))
    }

    // This is to create a `Password` instance but without fulfilling the password requirements
    pub(crate) fn fake(password: Secret<String>) -> Self {
        Password(password)
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        // Compare the exposed secrets in a controlled manner
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for Password {}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::quickcheck;

    #[test]
    fn test_valid_password() {
        let test_cases = [
            "password123",
            "12345678",
            "abcdefgh1",
            "abcdEFGH1",
            "abcdEFGH1!@#",
        ];
        for password in test_cases.iter() {
            let secret = Secret::new((*password).to_owned());
            assert!(
                Password::parse(secret).is_ok(),
                "Expected {} to be valid",
                password
            );
        }
    }

    #[test]
    fn test_invalid_password() {
        let test_cases = ["short", "noDigits", "1234567", ""];
        for password in test_cases.iter() {
            let secret = Secret::new((*password).to_owned());
            assert!(
                Password::parse(secret).is_err(),
                "Expected {} to be invalid",
                password
            );
        }
    }

    // Dumb tests just to familirize with the quickcheck crate
    quickcheck! {
            fn prop_valid_password_with_quickcheck(password: String) -> bool {
                let simple_check = password.len() >= 8 && password.chars().any(|c| c.is_digit(10));
                let secret = Secret::new(password);
                if simple_check {
                    Password::parse(secret).is_ok()
                } else {
                    true
                }
            }
    }
}
