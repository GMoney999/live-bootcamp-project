use async_trait::async_trait;

use crate::domain::Email;

#[async_trait]
pub trait EmailClient {
        async fn send_email(
                &self,
                recipient: &Email,
                subject: &str,
                content: &str,
        ) -> Result<(), String>;
}
