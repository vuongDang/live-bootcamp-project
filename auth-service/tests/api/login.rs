use crate::helpers::{app_signup, get_random_email};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use auth_service::{routes::TwoFactorLoginResponse, Email};
use reqwest::{cookie::CookieStore, Url};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn valid_login_without_2fa_returns_200() {
    let (mut app, email, password) = app_signup(false).await;

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
    app.cleanup().await;
}

#[tokio::test]
async fn malformed_login_returns_422() {
    let (mut app, email, password) = app_signup(false).await;
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
    app.cleanup().await;
}

#[tokio::test]
async fn invalid_login_returns_400() {
    let (mut app, email, password) = app_signup(false).await;

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
    app.cleanup().await;
}

#[tokio::test]
async fn incorrect_login_returns_401() {
    let (mut app, email, password) = app_signup(false).await;

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

    app.cleanup().await;
}

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let (mut app, email, pwd) = app_signup(true).await;

    // Define an expectation for the mock server
    Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
        .and(method("POST")) // Expect the HTTP method to be POST
        .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status
        .expect(1) // Expect this request to be made exactly once
        .mount(&app.email_server) // Mount this expectation on the mock email server
        .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding

    let login_body = serde_json::json!({
        "email": email,
        "password": pwd,
    });

    let response = app.post_login(&login_body).await;
    assert_eq!(
        response.status().as_u16(),
        206,
        "failed login for input: {:?}",
        login_body
    );

    let response_body: Result<TwoFactorLoginResponse, reqwest::Error> = response.json().await;
    assert!(
        response_body.is_ok(),
        "Missing response body: {:?}",
        response_body
    );
    let response_body = response_body.unwrap();
    let attempt_id_from_response = response_body.login_attempt_id;
    let attempt_id_from_store = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&email).unwrap())
        .await;
    let attempt_id_from_store = attempt_id_from_store.unwrap().1;
    assert_eq!(
        attempt_id_from_response,
        attempt_id_from_store.as_ref(),
        "login_attempt_id code from response and store do not match"
    );
    app.cleanup().await;
}
