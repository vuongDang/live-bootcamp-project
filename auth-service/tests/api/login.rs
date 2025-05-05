use crate::helpers::{app_signup, get_random_email};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::{cookie::CookieStore, Url};

#[tokio::test]
async fn valid_login_without_2fa_returns_200() {
    let (app, email, password) = app_signup().await;

    let login_body = serde_json::json!({
        "email": email,
        "password": password,
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed login for input: {:?}",
        login_body
    );

    let auth_cookie = response
        .cookies()
        .find(|c| c.name() == JWT_COOKIE_NAME)
        .expect("auth_cookie not found in response cookies");
    assert!(auth_cookie.value().len() > 0, "auth_cookie is empty");

    // Check if the cookie jar is updated with the auth cookie
    let jar_state = (*app.cookie_jar).cookies(&Url::parse(&app.address).unwrap());
    assert!(
        jar_state.is_some(),
        "cookie jar should be empty after logout"
    );
}

#[tokio::test]
async fn malformed_login_returns_422() {
    let (app, email, password) = app_signup().await;
    let test_cases = [
        serde_json::json!({
            "password": password,
        }),
        serde_json::json!({
            "email": email,
        }),
        serde_json::json!({}),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn invalid_login_returns_400() {
    let (app, email, password) = app_signup().await;

    let test_cases = [
        serde_json::json!({
            "email": "wrong_email",
            "password": password,
        }),
        serde_json::json!({
            "email": email,
            "password": "wrong_password",
        }),
        serde_json::json!({
            "email": "wrong_email",
            "password": "wrong_password",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn incorrect_login_returns_401() {
    let (app, email, password) = app_signup().await;

    let test_cases = [
        serde_json::json!({
            "email": get_random_email(),
            "password": password,
        }),
        serde_json::json!({
            "email": email,
            "password": "wrongpassword12346",
        }),
        serde_json::json!({
            "email": get_random_email(),
            "password": "wrong_password12345",
        }),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "failed for input: {:?}",
            test_case
        );
    }
}
