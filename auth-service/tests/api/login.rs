use crate::{LoginPayload, TestApp, TestResult};

#[tokio::test]
async fn login_returns_200() -> TestResult<()> {
        let app = TestApp::new().await?;
        let payload =
                LoginPayload::new("GSadeghi@admin.gov".to_string(), "Login password".to_string());

        let response = app.post_login(&payload).await?;
        assert_eq!(response.status().as_u16(), 200);

        Ok(())
}
