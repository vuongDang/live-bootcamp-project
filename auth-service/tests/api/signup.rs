use crate::helpers::TestApp;

#[tokio::test]
async fn signup_returns_auth_ui() {
    let app = TestApp::new().await;

    let response = app.post_signup().await;

    assert_eq!(response.status().as_u16(), 200);
}