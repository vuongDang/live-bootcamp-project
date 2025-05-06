use crate::helpers::app_signup_and_login;
use auth_service::{utils::constants::JWT_COOKIE_NAME, Email, LoginAttemptId, TwoFACode};
use reqwest::{cookie::CookieStore, Url};

#[tokio::test]
async fn should_return_200_if_valid_credentials_and_2fa_enabled() {
    let (app, email, _, _, login_attempt_id_from_response) = app_signup_and_login(true).await;
    let (two_fa_code, login_attempt_id) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    assert!(
        login_attempt_id_from_response.is_some(),
        "login attempt id is None"
    );
    assert_eq!(
        login_attempt_id_from_response.unwrap(),
        login_attempt_id.as_ref(),
        "login attempt id from response does not match the one in the store"
    );

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    println!("body: {:?}", body);

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed for input: {:?}",
        body
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
async fn returns_422_if_json_body_malformed() {
    let (app, email, _, _, login_attempt_id) = app_signup_and_login(true).await;

    let test_cases = vec![
        // Missing email
        serde_json::json!({
            "email": email,
            "loginAttemptId": login_attempt_id.clone().unwrap()
        }),
        serde_json::json!({}),
        serde_json::json!({
            "email": email,
            "2FACode": "random_code"
        }),
        serde_json::json!({
            "loginAttemptId": &login_attempt_id.clone().unwrap(),
            "2FACode": "random_code"
        }),
    ];

    for body in test_cases {
        let response = app.post_verify_2fa(&body).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "failed for input: {:?}",
            body
        );
    }
}

#[tokio::test]
async fn returns_400_if_invalid_input() {
    let (app, email, _, _, login_attempt_id) = app_signup_and_login(true).await;
    let login_attempt_id = login_attempt_id.clone().unwrap();

    let test_cases = vec![
        // Missing email
        serde_json::json!({
            "email": email,
            "loginAttemptId": &login_attempt_id,
            "2FACode": "invalid_code"
        }),
        serde_json::json!({
            "email": email,
            "loginAttemptId": "invalid_id",
            "2FACode": TwoFACode::new().as_ref(),
        }),
        serde_json::json!({
            "email": "invalid_email",
            "loginAttemptId": &login_attempt_id,
            "2FACode": TwoFACode::new().as_ref(),
        }),
    ];

    for body in test_cases {
        let response = app.post_verify_2fa(&body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "failed for input: {:?}",
            body
        );
    }
}

#[tokio::test]
async fn return_401_if_invalid_credentials() {
    for _ in 0..1 {
        let (app, email, _, _, _) = app_signup_and_login(true).await;
        let body = serde_json::json!({
            "email": email,
            "loginAttemptId": LoginAttemptId::new().as_ref(),
            "2FACode": TwoFACode::new().as_ref(),
        });
        let response = app.post_verify_2fa(&body).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "failed for input: {:?}",
            body
        );
    }
}

#[tokio::test]
async fn return_401_if_same_code_twice() {
    let (app, email, _, _, login_attempt_id_from_response) = app_signup_and_login(true).await;
    let (two_fa_code, login_attempt_id) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&Email::parse(&email).unwrap())
        .await
        .unwrap();
    assert!(
        login_attempt_id_from_response.is_some(),
        "login attempt id is None"
    );
    assert_eq!(
        login_attempt_id_from_response.unwrap(),
        login_attempt_id.as_ref(),
        "login attempt id from response does not match the one in the store"
    );

    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": login_attempt_id.as_ref(),
        "2FACode": two_fa_code.as_ref(),
    });
    println!("body: {:?}", body);

    let response = app.post_verify_2fa(&body).await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed for input: {:?}",
        body
    );

    // Try to use the same code again
    let response = app.post_verify_2fa(&body).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "failed for input: {:?}",
        body
    );
}
