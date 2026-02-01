use auth_service::{routes::SignupResponse, ErrorResponse};
use axum::response;

use crate::{get_random_email, SignupPayload, TestApp, TestResult};

#[tokio::test]
async fn should_return_201_if_valid_input() -> TestResult<()> {
        let app = TestApp::new().await?;
        let valid_input = serde_json::json!({
                "email": "valid@mail.com",
                "password": "ValidPassword123",
                "requires2FA": false
        });
        let res = app.post_signup(&valid_input).await;
        assert_eq!(res.status().as_u16(), 201);

        let expected_response = SignupResponse {
                message: "User created successfully!".to_string(),
        };
        assert_eq!(
                res.json::<SignupResponse>()
                        .await
                        .expect("Could not deserialize response body to SignupResponse"),
                expected_response
        );

        Ok(())
}

#[tokio::test]
async fn should_return_422_if_malformed_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let test_cases = [
                serde_json::json!({
                        "email": "valid@mail.com",
                        "password": "ValidPassword123"
                }),
                serde_json::json!({
                        "password": "ValidPassword123",
                        "requires2FA": true
                }),
                serde_json::json!({
                        "password": "ValidPassword123",
                        "requires2FA": true
                }),
                serde_json::json!({
                        "email": "valid@mail.com",
                        "requires2FA": true
                }),
                serde_json::json!({
                        "email": 123,
                        "password": "ValidPassword123",
                        "requires2FA": false
                }),
                serde_json::json!({
                        "email": "valid@mail.com",
                        "password": 123,
                        "requires2FA": false
                }),
                serde_json::json!({
                        "email": 123,
                        "password": 123,
                        "requires2FA": 123
                }),
        ];

        for test_case in test_cases.iter() {
                let response = app.post_signup(test_case).await;
                assert_eq!(response.status().as_u16(), 422, "Failed for input: {:?}", test_case);
        }

        Ok(())
}

#[tokio::test]
async fn should_return_400_if_invalid_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        // The signup route should return a 400 HTTP status code if an invalid input is sent.
        // The input is considered invalid if:
        // - The email is empty or does not contain '@'
        // - The password is less than 8 characters
        let test_cases = [
                // Invalid email
                serde_json::json!({
                        "email": "no at symbol and no dot",
                        "password": "ValidPassword123",
                        "requires2FA": false,
                }),
                // Invalid password
                serde_json::json!({
                        "email": "valid@mail.com",
                        "password": "2short",
                        "requires2FA": false,
                }),
                // Invalid email & password
                serde_json::json!({
                        "email": "no at symbol and no dot",
                        "password": "2short",
                        "requires2FA": false,
                }),
        ];

        // Create an array of invalid inputs. Then, iterate through the array and
        // make HTTP calls to the signup route. Assert a 400 HTTP status code is returned.
        for test_case in test_cases.iter() {
                let res = app.post_signup(&test_case).await;
                assert_eq!(res.status().as_u16(), 400, "Failed for input: {:?}", test_case);

                assert_eq!(
                        res.json::<ErrorResponse>()
                                .await
                                .expect("Could not deserialize response body to ErrorResponse")
                                .error,
                        "Invalid credentials".to_owned()
                );
        }

        Ok(())
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() -> TestResult<()> {
        // Call the signup route twice. The second request should fail with a 409 HTTP status code
        let app = TestApp::new().await?;

        let signup_payload = serde_json::json!({
                "email": "duplicate@mail.com",
                "password": "ValidPassword123",
                "requires2FA": false
        });

        // First time signing up
        app.post_signup(&signup_payload).await;
        // Second time signing up (duplicate email)
        let res = app.post_signup(&signup_payload).await;

        assert_eq!(res.status().as_u16(), 409);

        assert_eq!(
                res.json::<ErrorResponse>()
                        .await
                        .expect("Could not deserialize response body to ErrorResponse")
                        .error,
                "User already exists".to_owned()
        );

        Ok(())
}
