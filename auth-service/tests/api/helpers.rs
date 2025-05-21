use auth_service::app_state::*;
use auth_service::email_clients::postmark_email_client::PostmarkEmailClient;
use auth_service::get_postgres_pool;
use auth_service::routes::LoginResponse;
use auth_service::utils::constants::*;
use auth_service::Application;
use auth_service::Email;
use auth_service::PostgresUserStore;
use reqwest::cookie::Jar;
use reqwest::Client;
use secrecy::Secret;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::PgConnection;
use sqlx::PgPool;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use wiremock::matchers::method;
use wiremock::matchers::path;
use wiremock::Mock;
use wiremock::MockServer;
use wiremock::ResponseTemplate;

pub struct TestApp {
    pub address: String,
    pub cookie_jar: Arc<Jar>,
    pub http_client: reqwest::Client,
    pub banned_tokens: BannedTokenStoreType,
    pub two_fa_code_store: TwoFACodeStoreType,
    pub email_server: MockServer,
    pub db_name: String,
    cleanup_called: bool,
}

impl TestApp {
    pub async fn new() -> Self {
        let mut app_state = AppState::new_ps_redis().await;

        // Reconfigure the PostgreSQL database for testing and get db name
        let (db_pool, db_name) = configure_postgresql_test().await;
        app_state.user_store = Arc::new(tokio::sync::RwLock::new(PostgresUserStore::new(db_pool)));

        // Configure the email server
        let email_server = MockServer::start().await;
        let base_url = email_server.uri();
        let email_client = Arc::new(RwLock::new(configure_postmark_email_client(base_url)));
        app_state.email_client = email_client;

        let banned_tokens = app_state.banned_token_store.clone();
        let two_fa_code_store = app_state.two_fa_code_store.clone();
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
            banned_tokens,
            two_fa_code_store,
            db_name,
            email_server,
            cleanup_called: false,
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

    pub async fn post_verify_2fa<Body>(&self, body: &Body) -> reqwest::Response
    where
        Body: serde::Serialize,
    {
        self.http_client
            .post(&format!("{}/verify-2fa", &self.address))
            .json(body)
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

    pub async fn cleanup(&mut self) {
        // Cleanup the database
        delete_database(&self.db_name).await;
        self.cleanup_called = true;
    }
}

/// Implement the Drop trait for TestApp to clean up the database after tests
impl Drop for TestApp {
    fn drop(&mut self) {
        if !self.cleanup_called {
            panic!(
                "TestApp was not cleaned up properly. Please ensure to call cleanup() after tests."
            );
        }
    }
}

pub fn get_random_email() -> String {
    format!("{}@example.com", Uuid::new_v4())
}

pub async fn app_signup(with_2fa: bool) -> (TestApp, String, String) {
    let app = TestApp::new().await;
    let email = get_random_email();
    let password = "password123";

    let signup_body = serde_json::json!({
        "email": email.clone(),
        "password": password,
        "requires2FA": with_2fa
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
pub async fn app_signup_and_login(
    with_2fa: bool,
) -> (TestApp, String, String, Option<String>, Option<String>) {
    let (app, email, password) = app_signup(with_2fa).await;

    // Setup up email mock server if 2FA enabled
    if with_2fa {
        // Define an expectation for the mock server
        Mock::given(path("/email")) // Expect an HTTP request to the "/email" path
            .and(method("POST")) // Expect the HTTP method to be POST
            .respond_with(ResponseTemplate::new(200)) // Respond with an HTTP 200 OK status
            .expect(1) // Expect this request to be made exactly once
            .mount(&app.email_server) // Mount this expectation on the mock email server
            .await; // Await the asynchronous operation to ensure the mock server is set up before proceeding
    }

    let login_body = serde_json::json!({
        "email": email.clone(),
        "password": password.clone(),
    });

    let expected_code = if with_2fa { 206 } else { 200 };
    let response = app.post_login(&login_body).await;
    assert_eq!(
        response.status().as_u16(),
        expected_code,
        "failed login for input: {:?}",
        login_body
    );

    if !with_2fa {
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
        (
            app,
            email,
            password,
            Some(auth_cookie.value().to_string()),
            None,
        )
    } else {
        let login_response = response.json::<LoginResponse>().await.unwrap();
        let login_attempt_id: Option<String> = match login_response {
            LoginResponse::With2FA(response) => Some(response.login_attempt_id),
            _ => None,
        };
        (app, email, password, None, login_attempt_id)
    }
}

// Configure the PostgreSQL database for testing
// This function creates a new database for each test case to ensure isolation
// and runs the necessary migrations.
// It returns the database connection pool and the database name.
async fn configure_postgresql_test() -> (PgPool, String) {
    let postgresql_conn_url = DATABASE_URL.to_owned();

    // We are creating a new database for each test case, and we need to ensure each database has a unique name!
    let db_name = Uuid::new_v4().to_string();

    configure_database(&postgresql_conn_url, &db_name).await;

    let postgresql_conn_url_with_db = format!("{}/{}", postgresql_conn_url, db_name);

    // Create a new connection pool and return it
    let pool = get_postgres_pool(&postgresql_conn_url_with_db)
        .await
        .expect("Failed to create Postgres connection pool!");
    (pool, db_name)
}

async fn configure_database(db_conn_string: &str, db_name: &str) {
    // Create database connection
    let connection = PgPoolOptions::new()
        .connect(db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Create a new database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Connect to new database
    let db_conn_string = format!("{}/{}", db_conn_string, db_name);

    let connection = PgPoolOptions::new()
        .connect(&db_conn_string)
        .await
        .expect("Failed to create Postgres connection pool.");

    // Run migrations against new database
    sqlx::migrate!()
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
}

async fn delete_database(db_name: &str) {
    let postgresql_conn_url: String = DATABASE_URL.to_owned();

    let connection_options = PgConnectOptions::from_str(&postgresql_conn_url)
        .expect("Failed to parse PostgreSQL connection string");

    let mut connection = PgConnection::connect_with(&connection_options)
        .await
        .expect("Failed to connect to Postgres");

    // Kill any active connections to the database
    connection
        .execute(
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}'
                  AND pid <> pg_backend_pid();
        "#,
                db_name
            )
            .as_str(),
        )
        .await
        .expect("Failed to drop the database.");

    // Drop the database
    connection
        .execute(format!(r#"DROP DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to drop the database.");
}

fn configure_postmark_email_client(base_url: String) -> PostmarkEmailClient {
    let postmark_auth_token = Secret::new("auth_token".to_owned());

    let sender = Email::parse(&test::email_client::SENDER.to_owned()).unwrap();

    let http_client = Client::builder()
        .timeout(test::email_client::TIMEOUT)
        .build()
        .expect("Failed to build HTTP client");

    PostmarkEmailClient::new(base_url, sender, postmark_auth_token, http_client)
}
