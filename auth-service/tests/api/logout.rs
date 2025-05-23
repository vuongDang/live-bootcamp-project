use crate::helpers::{app_signup, app_signup_and_login};
use auth_service::utils::constants::JWT_COOKIE_NAME;
use reqwest::cookie::CookieStore;
use reqwest::Url;

#[tokio::test]
async fn should_return_200_if_jwt_is_valid() {
    let (mut app, _, _, jwt, _) = app_signup_and_login(false).await;
    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        200,
        "failed for logout with cookie jar: {:?}",
        app.cookie_jar
    );
    let jar_state = (*app.cookie_jar).cookies(&Url::parse(&app.address).unwrap());
    assert!(
        jar_state.is_none(),
        "cookie jar should be empty after logout"
    );
    let is_token_banned = app
        .banned_tokens
        .read()
        .await
        .is_token_banned(&jwt.unwrap())
        .await;
    assert!(is_token_banned.is_ok());
    assert!(is_token_banned.unwrap());
    app.cleanup().await;
}

#[tokio::test]
async fn should_return_400_if_jwt_cookie_missing() {
    // Signup but do not login
    let (mut app, _, _) = app_signup(false).await;

    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        400,
        "failed for logout with cookie jar: {:?}",
        app.cookie_jar
    );
    app.cleanup().await;
}

#[tokio::test]
async fn should_return_401_if_invalid_token() {
    // Signup but do not login
    let (mut app, _, _) = app_signup(false).await;

    // add invalid cookie
    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );
    let response = app.post_logout().await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "failed for logout with cookie jar: {:?}",
        app.cookie_jar
    );
    app.cleanup().await;
}
