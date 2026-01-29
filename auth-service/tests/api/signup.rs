use auth_service::routes::SignupResponse;

use crate::{get_random_email, SignupPayload, TestApp, TestResult};

#[tokio::test]
async fn should_return_201_if_valid_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let valid_input = serde_json::json!({
                "email": "valid@mail.com",
                "password": "password123",
                "requires2FA": false,
        });

        let res = app.post_signup(&valid_input).await;

        assert_eq!(res.status().as_u16(), 201);

        let expected_response = SignupResponse {
                message: "User created successfully!".to_string(),
        };

        assert_eq!(
                res.json::<SignupResponse>()
                        .await
                        .expect("Could not deserialize response body to UserBody"),
                expected_response
        );

        Ok(())
}

#[tokio::test]
async fn should_return_422_if_malformed_input() -> TestResult<()> {
        let app = TestApp::new().await?;

        let random_email = get_random_email();

        // TODO: add more malformed input test cases
        let test_cases = [serde_json::json!({
            "password": "password123",
            "requires2FA": true
        })];

        for test_case in test_cases.iter() {
                let response = app.post_signup(test_case).await;
                assert_eq!(response.status().as_u16(), 422, "Failed for input: {:?}", test_case);
        }

        Ok(())
}
