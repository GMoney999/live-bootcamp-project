use auth_service::{
        domain::BannedTokenStore,
        routes::{LoginPayload, SignupPayload, Verify2FAPayload, VerifyTokenPayload},
        services::{HashmapUserStore, HashsetBannedTokenStore},
        AppState, Application,
};
use axum_extra::extract::CookieJar;
use reqwest::cookie::Jar;
use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;

type TestAppResult = core::result::Result<reqwest::Response, Box<dyn std::error::Error>>;

pub struct TestApp {
        pub address: String,
        pub cookie_jar: Arc<Jar>,
        pub banned_token_store: Arc<RwLock<Box<dyn BannedTokenStore>>>,
        pub http_client: reqwest::Client,
}

impl TestApp {
        pub async fn new() -> Result<Self, Box<dyn Error>> {
                let app_state =
                        AppState::new(HashmapUserStore::new(), HashsetBannedTokenStore::new());

                let banned_token_store = app_state.banned_token_store.clone();

                let app = Application::build(app_state, "127.0.0.1:0").await?;

                let address = format!("http://{}", app.address.clone());

                #[allow(clippy::let_underscore_future)]
                let _ = tokio::spawn(app.run());

                let cookie_jar = Arc::new(Jar::default());

                let http_client = reqwest::Client::builder()
                        .cookie_provider(cookie_jar.clone())
                        .build()
                        .unwrap();

                Ok(TestApp {
                        address,
                        cookie_jar,
                        banned_token_store,
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

        pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
        where
                Body: serde::Serialize,
        {
                self.http_client
                        .post(format!("{}/login", &self.address))
                        .json(body)
                        .send()
                        .await
                        .expect("Failed to execute request")
        }

        pub async fn post_logout(&self) -> TestAppResult {
                let response =
                        self.http_client.post(format!("{}/logout", &self.address)).send().await?;
                Ok(response)
        }

        pub async fn post_verify_token<Body>(&self, body: &Body) -> TestAppResult
        where
                Body: serde::Serialize,
        {
                let response = self
                        .http_client
                        .post(format!("{}/verify-token", self.address))
                        .json(&body)
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
