use crate::{TestApp, TestResult, Verify2FAPayload};

#[tokio::test]
async fn verify_2fa_returns_200() -> TestResult<()> {
        let app = TestApp::new().await?;

        Ok(())
}
