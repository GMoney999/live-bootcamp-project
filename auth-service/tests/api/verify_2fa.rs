use auth_service::{
        domain::{Email, ErrorResponse},
        routes::TwoFactorAuthResponse,
        utils::constants::JWT_COOKIE_NAME,
};

use crate::{get_random_email, TestApp, TestResult};

async fn signup_and_login_with_2fa(
        app: &TestApp,
        email: &str,
        password: &str,
) -> TestResult<(String, String)> {
        let signup_payload = serde_json::json!({
                "email": email,
                "password": password,
                "requires2FA": true
        });
        let signup_response = app.post_signup(&signup_payload).await;
        assert_eq!(signup_response.status().as_u16(), 201, "Signup should succeed");

        let login_payload = serde_json::json!({
                "email": email,
                "password": password
        });
        let login_response = app.post_login(&login_payload).await;
        assert_eq!(login_response.status().as_u16(), 206, "Login should require 2FA");

        let two_fa_response = login_response
                .json::<TwoFactorAuthResponse>()
                .await
                .expect("Could not deserialize response body to TwoFactorAuthResponse");

        let parsed_email = Email::parse(email).expect("Email should be valid in test setup");
        let (_, code) = app
                .two_fa_code_store
                .read()
                .await
                .get_code(&parsed_email)
                .await
                .expect("2FA code should be present in store after login");

        Ok((two_fa_response.login_attempt_id, code.as_ref().to_owned()))
}

#[tokio::test]
async fn should_return_200_if_correct_code() -> TestResult<()> {
        // Make sure to assert the auth cookie gets set
        let app = TestApp::new().await?;

        let email = get_random_email();
        let password = "ValidPassword123";
        let (login_attempt_id, code) = signup_and_login_with_2fa(&app, &email, password).await?;

        let payload = serde_json::json!({
                "email": email,
                "loginAttemptId": login_attempt_id,
                "code": code
        });
        let response = app.post_verify_2fa(&payload).await?;

        assert_eq!(response.status().as_u16(), 200);

        let auth_cookie = response
                .cookies()
                .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
                .expect("JWT cookie should be set after successful 2FA verification");
        assert!(!auth_cookie.value().is_empty(), "JWT cookie value should not be empty");

        Ok(())
}

#[tokio::test]
async fn should_return_400_if_invalid_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let test_cases = [
                serde_json::json!({
                        "email": "invalid-email",
                        "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
                        "code": "123456"
                }),
                serde_json::json!({
                        "email": "valid@mail.com",
                        "loginAttemptId": "not-a-valid-uuid",
                        "code": "123456"
                }),
                serde_json::json!({
                        "email": "valid@mail.com",
                        "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000",
                        "code": "12ab56"
                }),
        ];

        for test_case in test_cases.iter() {
                let response = app.post_verify_2fa(test_case).await?;

                assert_eq!(
                        response.status().as_u16(),
                        400,
                        "Failed for input: {:?}",
                        test_case
                );
                assert_eq!(
                        response
                                .json::<ErrorResponse>()
                                .await
                                .expect("Could not deserialize response body to ErrorResponse")
                                .error,
                        "Invalid credentials".to_owned()
                );
        }

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() -> TestResult<()> {
        // For security reasons, if a user has successfully authenticated using a login attempt ID & 2FA code combination, then they should NOT be able to authenticate again using the same combination! In other words, 2FA codes should be used only once!
        let app = TestApp::new().await?;

        let email = get_random_email();
        let password = "ValidPassword123";
        let (login_attempt_id, code) = signup_and_login_with_2fa(&app, &email, password).await?;

        let payload = serde_json::json!({
                "email": email.clone(),
                "loginAttemptId": login_attempt_id.clone(),
                "code": code.clone()
        });

        let first_response = app.post_verify_2fa(&payload).await?;
        assert_eq!(first_response.status().as_u16(), 200, "First verification should succeed");

        let second_response = app.post_verify_2fa(&payload).await?;
        assert_eq!(
                second_response.status().as_u16(),
                401,
                "Second verification with the same code should fail"
        );

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_incorrect_credentials() -> TestResult<()> {
        let app = TestApp::new().await?;

        let email = get_random_email();
        let password = "ValidPassword123";
        let (login_attempt_id, code) = signup_and_login_with_2fa(&app, &email, password).await?;

        let wrong_code = if code == "000000" {
                "111111".to_owned()
        } else {
                "000000".to_owned()
        };

        let payload = serde_json::json!({
                "email": email,
                "loginAttemptId": login_attempt_id,
                "code": wrong_code
        });
        let response = app.post_verify_2fa(&payload).await?;

        assert_eq!(response.status().as_u16(), 401);

        Ok(())
}

#[tokio::test]
async fn should_return_401_if_old_code() -> TestResult<()> {
        // Call login twice. Then, attempt to call verify-fa with the 2FA code from the first login requet. This should fail.
        let app = TestApp::new().await?;

        let email = get_random_email();
        let password = "ValidPassword123";

        let (old_login_attempt_id, old_code) =
                signup_and_login_with_2fa(&app, &email, password).await?;

        let first_verify_payload = serde_json::json!({
                "email": email.clone(),
                "loginAttemptId": old_login_attempt_id.clone(),
                "code": old_code.clone()
        });
        let first_verify_response = app.post_verify_2fa(&first_verify_payload).await?;
        assert_eq!(
                first_verify_response.status().as_u16(),
                200,
                "First verification should succeed"
        );

        let second_login_payload = serde_json::json!({
                "email": email.clone(),
                "password": password
        });
        let second_login_response = app.post_login(&second_login_payload).await;
        assert_eq!(second_login_response.status().as_u16(), 206, "Second login should require 2FA");

        let _ = second_login_response
                .json::<TwoFactorAuthResponse>()
                .await
                .expect("Could not deserialize response body to TwoFactorAuthResponse");

        let old_verify_payload = serde_json::json!({
                "email": email,
                "loginAttemptId": old_login_attempt_id,
                "code": old_code
        });
        let old_code_response = app.post_verify_2fa(&old_verify_payload).await?;

        assert_eq!(old_code_response.status().as_u16(), 401);

        Ok(())
}

#[tokio::test]
async fn should_return_422_if_malformed_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let test_cases = [
                serde_json::json!({
                        "wrong field": "wrong value"
                }),
                serde_json::json!({
                        "email": "valid@mail.com",
                        "loginAttemptId": "550e8400-e29b-41d4-a716-446655440000"
                }),
                serde_json::json!({
                        "email": 123,
                        "loginAttemptId": 123,
                        "code": 123
                }),
        ];

        for test_case in test_cases.iter() {
                let response = app.post_verify_2fa(test_case).await?;
                assert_eq!(
                        response.status().as_u16(),
                        422,
                        "Failed for malformed input: {:?}",
                        test_case
                );
        }

        Ok(())
}
