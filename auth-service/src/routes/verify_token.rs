use crate::{error::AuthAPIError, utils::auth::validate_token, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    State(app): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let jwt = request.token;
    // Check if the token is valid
    validate_token(&jwt)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;
    // Check if the token is banned
    let is_token_banned = app
        .banned_token_store
        .read()
        .await
        .is_token_banned(&jwt)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;
    if is_token_banned {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}
