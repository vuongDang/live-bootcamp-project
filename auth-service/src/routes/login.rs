use crate::domain::data_stores::UserStoreError;
use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::utils::auth::generate_auth_cookie;
use crate::{error::AuthAPIError, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Email provided is not valid
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    // Password provided is not valid
    let password =
        Password::parse(&request.password).map_err(|_| AuthAPIError::InvalidCredentials)?;

    let validate_login_result = state
        .user_store
        .read()
        .await
        .validate_user(&email, &password)
        .await;
    validate_login_result.map_err(|err| match err {
        // Email provided is not in the database
        UserStoreError::UserNotFound => AuthAPIError::AuthenticationFailure,
        // Credentials provided do not match the ones in the database
        UserStoreError::InvalidCredentials => AuthAPIError::AuthenticationFailure,
        _ => AuthAPIError::UnexpectedError,
    })?;

    let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);
    Ok((updated_jar, StatusCode::OK.into_response()))
}
