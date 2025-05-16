use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    app_state::AppState, error::AuthAPIError, utils::auth::generate_auth_cookie, Email,
    LoginAttemptId, TwoFACode,
};

#[tracing::instrument(name = "verify_2fa", skip_all)]
pub async fn verify_2fa(
    State(app): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    // Parse and validate the email, login attempt ID, and 2FA code from the request
    let email = Email::parse(&request.email).map_err(|_| AuthAPIError::InvalidCredentials)?;
    let login_attempt_id = LoginAttemptId::parse(&request.login_attempt_id)
        .map_err(|_| AuthAPIError::InvalidCredentials)?;
    let two_fa_code =
        TwoFACode::parse(request.two_fa_code).map_err(|_| AuthAPIError::InvalidCredentials)?;

    // Retrieve the 2FA code and login attempt ID from the store
    let (code, id) = app
        .two_fa_code_store
        .read()
        .await
        .get_code(&email)
        .await
        .map_err(|_| AuthAPIError::AuthenticationFailure)?;

    if login_attempt_id != id || code != two_fa_code {
        return Err(AuthAPIError::AuthenticationFailure);
    }

    // Remove the 2FA code from the store after successful verification
    app.two_fa_code_store
        .write()
        .await
        .remove_code(&email)
        .await
        .map_err(|e| AuthAPIError::UnexpectedError(e.into()))?;

    // Set jwt cookie in the response
    let auth_cookie = generate_auth_cookie(&email).map_err(AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);

    Ok((updated_jar, StatusCode::OK))
}

#[derive(Deserialize)]
pub struct Verify2FARequest {
    email: String,
    #[serde(rename = "loginAttemptId")]
    login_attempt_id: String,
    #[serde(rename = "2FACode")]
    two_fa_code: String,
}
