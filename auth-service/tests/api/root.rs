use crate::{TestApp, TestResult};

#[tokio::test]
async fn root_returns_auth_ui() -> TestResult<()> {
        let app = TestApp::new().await?;
        let response = app.get_login_or_signup().await?;
        assert_eq!(response.status().as_u16(), 200);

        Ok(())
}
