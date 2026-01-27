use crate::{TestApp, TestResult, VerifyTokenPayload};

#[tokio::test]
async fn verify_token_returns_200() -> TestResult<()> {
        let app = TestApp::new().await?;
        let token = VerifyTokenPayload::new("abc123".to_string());
        let response = app.post_verify_token(&token).await?;
        assert_eq!(response.status().as_u16(), 200);

        Ok(())
}
