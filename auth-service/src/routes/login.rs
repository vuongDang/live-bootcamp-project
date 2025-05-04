use crate::domain::data_stores::UserStoreError;
use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::{error::AuthAPIError, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let res = state
        .user_store
        .read()
        .await
        .validate_user(&email, &password)
        .await;
    match res {
        Ok(_) => Ok((StatusCode::OK, Json("Login successful"))),
        Err(err) => Err(match err {
            UserStoreError::UserNotFound => AuthAPIError::AuthenticationFailure,
            UserStoreError::InvalidCredentials => AuthAPIError::AuthenticationFailure,
            _ => AuthAPIError::UnexpectedError,
        }),
    }
}
