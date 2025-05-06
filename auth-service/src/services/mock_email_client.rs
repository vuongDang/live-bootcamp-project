use crate::domain::{email::Email, EmailClient};

#[derive(Debug, Clone, Default)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), String> {
        // Our mock email client will simply log the recipient, subject, and content to standard output
        println!(
            "Sending email to {} with\n\n[SUBJECT]: {}\n[CONTENT]: {}",
            recipient.as_ref(),
            subject,
            content
        );

        Ok(())
    }
}
