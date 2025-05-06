use auth_service::utils::constants::test;
use auth_service::utils::constants::JWT_COOKIE_NAME;
use auth_service::Application;
use reqwest::cookie::Jar;
use std::sync::Arc;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app_state = auth_service::AppState::default();
        let app = Application::build(app_state, test::APP_ADDRESS)
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        // Run the auth service in a separate async task
        // to avoid blocking the main test thread.
        #[allow(clippy::let_underscore_future)]
        let _ = tokio::spawn(app.run());

        let cookie_jar = Arc::new(Jar::default());
        // Create a Reqwest http client instance
        let http_client = reqwest::Client::builder()
            .cookie_provider(cookie_jar.clone())
            .build()
            .expect("Failed to build HTTP client");

        let test_app = TestApp {
            address,
            cookie_jar,
            http_client,
        };

        test_app
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_signup<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_2fa(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .send()
            .await
            .expect("Failed to execute request.")
    }

    pub async fn post_verify_token<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-token", &self.address))
            .json(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn app_signup() -> (TestApp, String, String) {
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

/// This function will sign up a user and then log them in, returning the app instance, email, password, and auth cookie.
pub async fn app_signup_and_login() -> (TestApp, String, String, String) {
    let (app, email, password) = app_signup().await;

    let login_body = serde_json::json!({
        "email": email.clone(),
        "password": password.clone(),
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

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}={}; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME,
            auth_cookie.value()
        ),
        &reqwest::Url::parse(&app.address).expect("Failed to parse URL"),
    );

    (app, email, password, auth_cookie.value().to_string())
}
