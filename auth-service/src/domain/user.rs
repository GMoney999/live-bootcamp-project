use crate::domain::{email::Email, password::Password};

#[derive(Debug, Clone, PartialEq)]
pub struct User {
        email: Email,
        password: Password,
        requires_2fa: bool,
}
impl User {
        pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
                Self {
                        email,
                        password,
                        requires_2fa,
                }
        }
        pub fn email(&self) -> &Email {
                &self.email
        }
        pub fn email_str(&self) -> &str {
                self.email.as_ref()
        }
        pub fn email_to_owned(&self) -> Email {
                self.email.clone()
        }
        pub fn password(&self) -> &Password {
                &self.password
        }
        pub fn password_str(&self) -> &str {
                self.password.as_ref()
        }
        pub fn password_to_owned(&self) -> Password {
                self.password.clone()
        }
        pub fn requires_2fa(&self) -> bool {
                self.requires_2fa
        }
}
