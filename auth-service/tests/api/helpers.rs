use auth_service::{
        routes::{
                LoginPayload, LogoutPayload, SignupPayload, Verify2FAPayload, VerifyTokenPayload,
        },
        Application,
};
use std::error::Error;

type TestAppResult = core::result::Result<reqwest::Response, Box<dyn std::error::Error>>;

pub struct TestApp {
        address: String,
        http_client: reqwest::Client,
}

impl TestApp {
        pub async fn new() -> Result<Self, Box<dyn Error>> {
                let app = Application::build("127.0.0.1:0").await?;
                let address = format!("http://{}", app.address.clone());

                #[allow(clippy::let_underscore_future)]
                let _ = tokio::spawn(app.run());

                let http_client = reqwest::Client::new();

                Ok(TestApp {
                        address,
                        http_client,
                })
        }

        pub async fn get_login_or_signup(&self) -> TestAppResult {
                let response = self.http_client.get(format!("{}/", &self.address)).send().await?;
                Ok(response)
        }

        pub async fn post_verify_2fa(&self, payload: &Verify2FAPayload) -> TestAppResult {
                let response = self
                        .http_client
                        .post(format!("{}/verify-2fa", &self.address))
                        .json(&payload)
                        .send()
                        .await?;
                Ok(response)
        }

        pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
        where
                Body: serde::Serialize,
        {
                self.http_client
                        .post(format!("{}/signup", &self.address))
                        .json(body)
                        .send()
                        .await
                        .expect("Failed to execute request")
        }

        pub async fn post_login(&self, creds: &LoginPayload) -> TestAppResult {
                let response = self
                        .http_client
                        .post(format!("{}/login", &self.address))
                        .json(&creds)
                        .send()
                        .await?;
                Ok(response)
        }

        pub async fn post_logout(&self, payload: &LogoutPayload) -> TestAppResult {
                let response = self
                        .http_client
                        .post(format!("{}/logout", &self.address))
                        .json(&payload)
                        .send()
                        .await?;
                Ok(response)
        }

        pub async fn post_verify_token(&self, token: &VerifyTokenPayload) -> TestAppResult {
                let response = self
                        .http_client
                        .post(format!("{}/verify-token", self.address))
                        .json(&token)
                        .send()
                        .await?;
                Ok(response)
        }
}

pub fn get_random_email() -> String {
        format!("{}@example.com", uuid::Uuid::new_v4())
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Token {
        pub value: String,
}
