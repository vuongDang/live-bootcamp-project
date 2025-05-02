use auth_service::Application;

#[tokio::main]
async fn main() {
    let state = auth_service::AppState::new(
        auth_service::UserStoreType::default(),
    );
    let app = Application::build(state, "0.0.0.0:3000").await.expect("Failed to build the app");
    app.run().await.expect("Failed to run the app");
}

