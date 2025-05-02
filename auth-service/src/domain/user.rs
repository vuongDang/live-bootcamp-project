#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct User {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) requires_2fa: bool,  
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
        }
    }
}