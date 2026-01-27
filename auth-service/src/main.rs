use auth_service::Application;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
        let app = Application::build("0.0.0.0:3000").await.expect("failed to build Application");

        app.run().await.expect("failed to run application");
        Ok(())
}
