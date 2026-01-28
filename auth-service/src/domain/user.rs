#[derive(Debug, Clone, PartialEq)]
pub struct User {
        email: String,
        password: String,
        requires_2fa: bool,
}
impl User {
        pub fn new<S: Into<String>>(email: S, password: S, requires_2fa: bool) -> Self {
                let (email, password): (String, String) = (email.into(), password.into());

                Self {
                        email,
                        password,
                        requires_2fa,
                }
        }
        pub fn email(&self) -> &String {
                &self.email
        }
        pub fn email_to_owned(&self) -> String {
                self.email.clone()
        }
        pub fn password(&self) -> &String {
                &self.password
        }
        pub fn password_to_owned(&self) -> String {
                self.password.clone()
        }
        pub fn requires_2fa(&self) -> bool {
                self.requires_2fa
        }
}
