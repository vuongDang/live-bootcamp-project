use crate::{error::AuthAPIError, utils::auth::validate_token};
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct VerifyTokenRequest {
    token: String,
}

pub async fn verify_token(
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let jwt = request.token;
    validate_token(&jwt)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;
    Ok(StatusCode::OK.into_response())
}
