use auth_service::{services::hashmap_user_store::HashmapUserStore, AppState, Application};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
        let store = Arc::new(RwLock::new(HashmapUserStore::new()));

        let app_state = AppState::new(store);

        let app = Application::build(app_state, "0.0.0.0:3000")
                .await
                .expect("failed to build Application");

        app.run().await.expect("failed to run application");
        Ok(())
}
