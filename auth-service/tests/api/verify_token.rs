use auth_service::{
        routes::{SignupPayload, LoginPayload, VerifyTokenPayload},
        utils::constants::JWT_COOKIE_NAME,
};

use crate::{TestApp, TestResult};

#[tokio::test]
async fn should_return_200_valid_token() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Create and signup a user
        let email = "verify_token@example.com".to_string();
        let password = "ValidPassword123".to_string();
        let signup = SignupPayload::new(email.clone(), password.clone(), false);
        let _ = app.post_signup(&signup).await;

        // Login to get a valid JWT token
        let login = LoginPayload::new(email, password);
        let response = app.post_login(&login).await;
        assert_eq!(response.status().as_u16(), 200, "Login should succeed");

        // Extract the JWT token from the cookie
        let auth_cookie = response
                .cookies()
                .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
                .expect("JWT cookie should be present");

        let token = auth_cookie.value().to_string();

        // Verify the token
        let verify_payload = VerifyTokenPayload::new(token);
        let response = app.post_verify_token(&verify_payload).await?;

        assert_eq!(response.status().as_u16(), 200, "Valid token should return 200");

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_invalid_token() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Try to verify an invalid token
        let invalid_token = "invalid.jwt.token".to_string();
        let verify_payload = VerifyTokenPayload::new(invalid_token);
        let response = app.post_verify_token(&verify_payload).await?;

        assert_eq!(
                response.status().as_u16(),
                401,
                "Invalid token should return 401"
        );

        Ok(())
}

#[tokio::test]
async fn should_return_422_if_malformed_input() -> TestResult<()> {
        let app = TestApp::new().await?;
        let req = serde_json::json!({
                "wrong field": "wrong value"
        });
        let res = app.post_verify_token(&req).await?;

        assert_eq!(res.status().as_u16(), 422);

        Ok(())
}
