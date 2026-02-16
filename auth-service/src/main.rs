use auth_service::{
        configure_postgresql,
        domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore},
        services::{
                hashmap_two_fa_code_store::HashmapTwoFACodeStore, HashmapUserStore,
                HashsetBannedTokenStore, MockEmailClient,
        },
        utils::constants::prod,
        AppState, Application,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
        let user_store: Arc<RwLock<Box<dyn UserStore + Send + Sync>>> =
                Arc::new(RwLock::new(Box::new(HashmapUserStore::new())));
        let banned_token_store: Arc<RwLock<Box<dyn BannedTokenStore + Send + Sync>>> =
                Arc::new(RwLock::new(Box::new(HashsetBannedTokenStore::new())));
        let two_fa_code_store: Arc<RwLock<Box<dyn TwoFACodeStore + Send + Sync>>> =
                Arc::new(RwLock::new(Box::new(HashmapTwoFACodeStore::new())));
        let email_client: Arc<dyn EmailClient + Send + Sync> = Arc::new(MockEmailClient);

        let pg_pool = configure_postgresql().await;

        let app_state =
                AppState::new(user_store, banned_token_store, two_fa_code_store, email_client);

        let app = Application::build(app_state, prod::APP_ADDRESS)
                .await
                .expect("failed to build Application");

        app.run().await.expect("failed to run application");
        Ok(())
}
