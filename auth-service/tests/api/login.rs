use crate::helpers::{get_random_email, TestApp};

async fn app_signup() -> (TestApp, String, String) {
    let app = TestApp::new().await;
    let email = get_random_email();
    let password = "password123";

    let signup_body = serde_json::json!({
        "email": email.clone(),
        "password": password,
        "requires2FA": true
    });

    let response = app.post_signup(&signup_body).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "failed signup for input: {:?}",
       signup_body 
    );
    (app, email, password.to_string())
}

#[tokio::test]
async fn valid_login_returns_200() {
    let (app, email, password) = app_signup().await;
    
    let login_body = serde_json::json!({
        "email": email,
        "password": password,
    });
    let response = app.post_login(&login_body).await;
    assert_eq!(response.status().as_u16(), 200, "failed login for input: {:?}", login_body);
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
        serde_json::json!({
                  }),
    ];
    
    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(response.status().as_u16(), 422, "failed for input: {:?}", test_case);
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
        assert_eq!(response.status().as_u16(), 400, "failed for input: {:?}", test_case);
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
        assert_eq!(response.status().as_u16(), 401, "failed for input: {:?}", test_case);
    }
}
