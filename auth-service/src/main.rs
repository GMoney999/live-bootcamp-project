use auth_service::{
        services::{HashmapUserStore, HashsetBannedTokenStore},
        utils::constants::prod,
        AppState, Application,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
        let app_state = AppState::new(HashmapUserStore::new(), HashsetBannedTokenStore::new());

        let app = Application::build(app_state, prod::APP_ADDRESS)
                .await
                .expect("failed to build Application");

        app.run().await.expect("failed to run application");
        Ok(())
}
