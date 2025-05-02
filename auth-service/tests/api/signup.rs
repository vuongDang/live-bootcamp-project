use crate::helpers::{get_random_email, TestApp};
use auth_service::error::ErrorResponse;

#[tokio::test]
async fn should_return_422_if_malformed_input() {
    let app = TestApp::new().await;

    let random_email = get_random_email();

    let test_cases = [
        serde_json::json!({
            "password": "password123",
            "requires2fa": true
        }),
        serde_json::json!({
            "email": random_email,
        }),
        serde_json::json!({
            "email": random_email,
            "requires2fa": true
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_201_if_valid_input() {
    let app: TestApp = TestApp::new().await;
    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "password123",
            "requires2FA": false
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "fdjskljfejzi1234",
            "requires2FA": false
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            201,
            "failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn should_return_400_if_invalid_input() {
    let app: TestApp = TestApp::new().await;
    let test_cases = [
        serde_json::json!({
            "email": "foo",
            "password": "password123",
            "requires2FA": true
        }),
        serde_json::json!({
            "email": "",
            "password": "password123",
            "requires2FA": false
        }),
        serde_json::json!({
            "email": "foo@toto.com",
            "password": "5char",
            "requires2FA": false
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "failed for input: {:?}",
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
}

#[tokio::test]
async fn should_return_409_if_email_already_exists() {
    let app: TestApp = TestApp::new().await;
    let test_case = serde_json::json!({
        "email": "foo@toto.com",
        "password": "password123",
        "requires2FA": true
    });

    let response = app.post_signup(&test_case).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "failed for input: {:?}",
        test_case
    );

    let response = app.post_signup(&test_case).await;
    assert_eq!(
        response.status().as_u16(),
        409,
        "failed for input: {:?}",
        test_case
    );
    
     assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "User already exists".to_owned()
    );
}
