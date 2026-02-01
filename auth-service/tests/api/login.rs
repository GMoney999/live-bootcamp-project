use crate::{get_random_email, TestApp, TestResult};
use auth_service::{utils::constants::JWT_COOKIE_NAME, ErrorResponse};

#[tokio::test]
async fn should_return_201_if_valid_credentials_and_2fa_disabled() -> TestResult<()> {
        let app = TestApp::new().await?;
        let random_email = get_random_email();
        let signup_payload = serde_json::json!({
                "email": random_email.clone(),
                "password": "ValidPassword123",
                "requires2FA": false
        });
        let res = app.post_signup(&signup_payload).await;
        assert_eq!(res.status().as_u16(), 201);

        let login_payload = serde_json::json!({
                "email": random_email,
                "password": "ValidPassword123"
        });
        let res = app.post_login(&login_payload).await;

        let auth_token = res
                .cookies()
                .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
                .expect("Failed to find jwt token cookie");

        assert!(!auth_token.name().is_empty());

        Ok(())
}

#[tokio::test]
async fn should_return_400_if_invalid_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let test_cases = [
                serde_json::json!({
                        "email": "valid@mail.com",
                        "password": "2short"
                }),
                serde_json::json!({
                        "email": "invalid email",
                        "password": "ValidPassword123"
                }),
                serde_json::json!({
                        "email": "invalid email",
                        "password": "2short"
                }),
        ];

        for test_case in test_cases.iter() {
                let response = app.post_login(test_case).await;
                assert_eq!(response.status().as_u16(), 400);
                assert_eq!(
                        response.json::<ErrorResponse>()
                                .await
                                .expect("Could not deserialize response body to ErrorResponse")
                                .error,
                        "Invalid credentials"
                );
        }

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() -> TestResult<()> {
        let app = TestApp::new().await?;
        let email = "valid@mail.com".to_string();
        let password = "ValidPassword123".to_string();
        let other_password = "ValidPassword456".to_string();

        let signup_payload = serde_json::json!({
                "email": email.clone(),
                "password": password.clone(),
                "requires2FA": false
        });

        let _ = app.post_signup(&signup_payload).await;

        let login = serde_json::json!({
                "email": email,
                "password": other_password
        });

        let res = app.post_login(&login).await;

        assert_eq!(res.status().as_u16(), 401);
        assert_eq!(
                res.json::<ErrorResponse>()
                        .await
                        .expect("Could not deserialize response body to ErrorResponse")
                        .error,
                "Unauthorized"
        );

        Ok(())
}

#[tokio::test]
async fn should_return_422_if_malformed_credentials() -> TestResult<()> {
        let app = TestApp::new().await?;

        let payload = serde_json::json!({
                "wrong field": "wrong value"
        });
        let response = app.post_login(&payload).await;
        assert_eq!(response.status().as_u16(), 422);

        Ok(())
}
