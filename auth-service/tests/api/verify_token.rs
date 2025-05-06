use crate::helpers::app_signup_and_login;
use auth_service::utils::auth::generate_auth_cookie;
use auth_service::Email;

#[tokio::test]
async fn should_return_200_if_jwt_is_valid() {
    let (app, _, _, jwt, _) = app_signup_and_login(false).await;
    let body = serde_json::json!({
        "token": jwt,
    });

    let response = app.post_verify_token(&body).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed for verify_token with cookie jar: {:?}",
        app.cookie_jar
    );
}

#[tokio::test]
async fn should_return_422_if_jwt_cookie_missing() {
    // Signup but do not login
    let (app, _, _, _, _) = app_signup_and_login(false).await;
    let test_cases = [
        serde_json::json!({
            "ton": "invalid_token",
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_token(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "failed for verify_token with cookie jar: {:?}",
            app.cookie_jar
        );
    }
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    // Signup but do not login
    let (app, email, _, jwt, _) = app_signup_and_login(false).await;

    // Ban the jwt token
    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed for logout with cookie jar: {:?}",
        app.cookie_jar
    );

    let test_cases = [
        serde_json::json!({
            "token": "invalid_token",
        }),
        serde_json::json!({
            "token": generate_auth_cookie(&Email::parse(&email).unwrap()).unwrap().to_string(),
        }),
        // jwt that was banned
        serde_json::json!({
            "token": jwt,
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_token(&test_case).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "failed for verify_token with cookie jar: {:?}",
            app.cookie_jar
        );
    }
}
