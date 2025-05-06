use crate::domain::email::Email;

// This trait represents the interface all concrete email clients should implement
#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String>;
}

pub fn two_fa_login_email_template(email: &Email, code: &str) -> (String, String) {
    let subject = "Your 2FA Code";
    let content = format!(
        "Hello {},\n\nYour 2FA code is: {}\n\nThank you!",
        email.as_ref(),
        code
    );
    (subject.to_string(), content)
}
