use crate::{TestApp, TestResult, Verify2FAPayload};

#[tokio::test]
async fn verify_2fa_returns_200() -> TestResult<()> {
        let app = TestApp::new().await?;
        let payload = Verify2FAPayload::new("".to_string(), "".to_string(), "".to_string());
        let response = app.post_verify_2fa(&payload).await?;
        assert_eq!(response.status().as_u16(), 200);

        Ok(())
}
