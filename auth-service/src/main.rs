use auth_service::{
        domain::{BannedTokenStore, EmailClient, TwoFACodeStore, UserStore},
        get_banned_token_store, get_email_client, get_two_fa_code_store, get_user_store,
        init_postgres_pool,
        services::data_stores::{
                postgres_user_store::PostgresUserStore, HashmapTwoFACodeStore, HashmapUserStore,
                HashsetBannedTokenStore, MockEmailClient,
        },
        utils::constants::prod,
        AppState, AppStateBuilder, Application,
};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
        color_eyre::install()?;

        let pg_pool = init_postgres_pool().await;

        let user_store = get_user_store(pg_pool);
        let banned_token_store = get_banned_token_store();
        let two_fa_code_store = get_two_fa_code_store();
        let email_client = get_email_client();

        let app_state = AppStateBuilder::new()
                .user_store(user_store)
                .banned_token_store(banned_token_store)
                .two_fa_code_store(two_fa_code_store)
                .email_client(email_client)
                .build();

        let app = Application::build(app_state, prod::APP_ADDRESS)
                .await
                .expect("failed to build Application");

        app.run().await.expect("failed to run application");
        Ok(())
}
