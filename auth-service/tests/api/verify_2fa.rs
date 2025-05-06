use crate::helpers::{app_signup, app_signup_and_login, TestApp};

#[tokio::test]
async fn should_return_206_if_valid_credentials_and_2fa_enabled() {
    let (app, email, _) = app_signup(true).await;
    let body = serde_json::json!({
        "email": email,
        "loginAttemptId": "",
        "2FACode": ""
    });

    // let response = app.post_verify_2fa(&body).await;
    // assert_eq!(
    //     response.status().as_u16(),
    //     206,
    //     "failed for input: {:?}",
    //     body
    // );
}
