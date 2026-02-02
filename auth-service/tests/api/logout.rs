use auth_service::{
        domain::ErrorResponse,
        routes::{LoginPayload, SignupPayload},
        utils::constants::JWT_COOKIE_NAME,
};
use reqwest::Url;

use crate::{TestApp, TestResult};

#[tokio::test]
async fn should_return_200_if_valid_jwt_cookie() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Create and signup a user
        let email = "logout@example.com".to_string();
        let password = "ValidPassword123".to_string();
        let signup = SignupPayload::new(email.clone(), password.clone(), false);
        let _ = app.post_signup(&signup).await;

        // Login to get a valid JWT cookie
        let login = LoginPayload::new(email, password);
        let response = app.post_login(&login).await;
        assert_eq!(response.status().as_u16(), 200, "Login should succeed");

        // Logout should succeed
        let response = app.post_logout().await?;
        assert_eq!(response.status().as_u16(), 200, "Logout should return 200");

        Ok(())
}

#[tokio::test]
async fn should_return_400_if_cookie_not_found() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Try to logout without logging in (no cookie)
        let response = app.post_logout().await?;

        assert_eq!(response.status().as_u16(), 400, "Should return 400 if no cookie");

        let error_response = response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse");

        assert_eq!(error_response.error, "Missing JWT auth token");

        Ok(())
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Create and signup a user
        let email = "logout_twice@example.com".to_string();
        let password = "ValidPassword123".to_string();
        let signup = SignupPayload::new(email.clone(), password.clone(), false);
        let _ = app.post_signup(&signup).await;

        // Login to get a valid JWT cookie
        let login = LoginPayload::new(email, password);
        let response = app.post_login(&login).await;
        assert_eq!(response.status().as_u16(), 200, "Login should succeed");

        // First logout should succeed
        let response = app.post_logout().await?;
        assert_eq!(response.status().as_u16(), 200, "First logout should succeed");

        // Second logout should fail (cookie removed after first logout)
        let response = app.post_logout().await?;
        assert_eq!(response.status().as_u16(), 400, "Second logout should return 400 (no cookie)");

        let error_response = response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse");

        assert_eq!(error_response.error, "Missing JWT auth token");

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_invalid_token() -> TestResult<()> {
        let app = TestApp::new().await?;

        // Add an invalid JWT cookie
        app.cookie_jar.add_cookie_str(
                &format!(
                        "{}=invalid_token; HttpOnly; SameSite=Lax; Secure; Path=/",
                        JWT_COOKIE_NAME
                ),
                &Url::parse(&app.address).expect("Failed to parse URL"),
        );

        // Try to logout with invalid token
        let response = app.post_logout().await?;

        assert_eq!(response.status().as_u16(), 401, "Should return 401 for invalid token");

        let error_response = response
                .json::<ErrorResponse>()
                .await
                .expect("Could not deserialize response body to ErrorResponse");

        assert_eq!(error_response.error, "Invalid JWT auth token");

        Ok(())
}
