use auth_service::utils::constants::prod;
use auth_service::utils::tracing::init_tracing;
use auth_service::Application;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");
    init_tracing().expect("Failed to initialize tracing");
    let state = auth_service::app_state::AppState::new_ps_redis().await;
    let app = Application::build(state, prod::APP_ADDRESS)
        .await
        .expect("Failed to build the app");
    app.run().await.expect("Failed to run the app");
}
