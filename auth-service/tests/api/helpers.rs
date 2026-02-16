use auth_service::{
        domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore},
        routes::{LoginPayload, SignupPayload, Verify2FAPayload, VerifyTokenPayload},
        services::{
                hashmap_two_fa_code_store::HashmapTwoFACodeStore, HashmapUserStore,
                HashsetBannedTokenStore, MockEmailClient,
        },
        AppState, Application, BannedTokenStoreType, EmailClientType, TwoFACodeStoreType,
};
use axum_extra::extract::CookieJar;
use reqwest::cookie::Jar;
use std::{error::Error, sync::Arc};
use tokio::sync::RwLock;

type TestAppResult = core::result::Result<reqwest::Response, Box<dyn std::error::Error>>;

pub struct TestApp {
        pub address: String,
        pub cookie_jar: Arc<Jar>,
        pub banned_token_store: BannedTokenStoreType,
        pub two_fa_code_store: TwoFACodeStoreType,
        pub email_client: EmailClientType,
        pub http_client: reqwest::Client,
}

impl TestApp {
        pub async fn new() -> Result<Self, Box<dyn Error>> {
                let user_store: Arc<RwLock<Box<dyn UserStore + Send + Sync>>> =
                        Arc::new(RwLock::new(Box::new(HashmapUserStore::new())));
                let banned_token_store: Arc<RwLock<Box<dyn BannedTokenStore + Send + Sync>>> =
                        Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::new())));
                let two_fa_code_store: Arc<RwLock<Box<dyn TwoFACodeStore + Send + Sync>>> =
                        Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::new())));
                let email_client: Arc<dyn EmailClient + Send + Sync> = Arc::new(MockEmailClient);

                let app_state = AppState::new(
                        user_store,
                        banned_token_store.clone(),
                        two_fa_code_store.clone(),
                        email_client.clone(),
                );

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
                        two_fa_code_store,
                        email_client,
                        http_client,
                })
        }

        pub async fn get_login_or_signup(&self) -> TestAppResult {
                let response = self.http_client.get(format!("{}/", &self.address)).send().await?;
                Ok(response)
        }

        pub async fn post_verify_2fa<Body>(&self, payload: &Body) -> TestAppResult
        where
                Body: serde::Serialize,
        {
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
