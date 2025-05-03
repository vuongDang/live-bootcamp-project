#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Password(pub String);

impl Password {
    pub fn parse(password: &str) -> Result<Self, String> {
        if is_valid_password(password) {
            Ok(Password(password.to_string()))
        } else {
            Err("Password not valid".to_string())
        }
    }
}

// Password must be at least 8 characters long and contain at least one digit
fn is_valid_password(password: &str) -> bool {
    password.len() >= 8 && password.chars().any(|c| c.is_digit(10))
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
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
            assert!(
                Password::parse(password).is_ok(),
                "Expected {} to be valid",
                password
            );
        }
    }

    #[test]
    fn test_invalid_password() {
        let test_cases = ["short", "noDigits", "1234567", ""];
        for password in test_cases.iter() {
            assert!(
                Password::parse(password).is_err(),
                "Expected {} to be invalid",
                password
            );
        }
    }

    // Dumb tests just to familirize with the quickcheck crate
    quickcheck! {
            fn prop_valid_password_with_quickcheck(password: String) -> bool {
                let simple_check = password.len() >= 8 && password.chars().any(|c| c.is_digit(10));
                if simple_check {
                    Password::parse(&password).is_ok()
                } else {
                    true
                }
            }
    }
}
