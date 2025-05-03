#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Email(pub String);

impl Email {
    pub fn parse(email: &str) -> Result<Self, String> {
        if is_valid_email(email) {
            Ok(Email(email.to_string()))
        } else { 
            Err("Email not valid".to_string())
        } 
    }
}

// Simple regex for email validation
pub(crate) fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Failed to compile regex");
    re.is_match(email)
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use quickcheck::quickcheck;

    #[test]
    fn test_valid_email_with_fake_crate() {
        let test_cases = fake::vec![String as SafeEmail(); 50];
        for email in test_cases.iter() {
            assert!(
                Email::parse(email).is_ok(),
                "Expected {} to be valid",
                email
            );
        }
    }

    #[test]
    fn test_invalid_email() {
        let test_cases = [
            "foo",
            "foo@toto",
            "@toto.com",
            "foo@.com",
            "foo@toto.c",
            // "foo@toto..com", not handled by our regex
            "",
        ];
        for email in test_cases.iter() {
            assert!(
                Email::parse(email).is_err(),
                "Expected {} to be invalid",
                email
            );
        }
    }

    quickcheck! {
        fn test_invalid_email_with_quickcheck(email: String) -> bool {
            let simple_check = email.is_empty() || !email.contains("@") || !email.contains(".");
            if simple_check {
                is_valid_email(&email) == false
            } else {
                true
            }
        }
    }
}
