use auth_service::utils::constants::prod;
use auth_service::Application;

#[tokio::main]
async fn main() {
    let state = auth_service::app_state::AppState::default();
    let app = Application::build(state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build the app");
    app.run().await.expect("Failed to run the app");
}
