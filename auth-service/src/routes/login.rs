use crate::domain::data_stores::{LoginAttemptId, TwoFACode, UserStoreError};
use crate::domain::email::Email;
use crate::domain::password::Password;
use crate::domain::EmailClient;
use crate::utils::auth::generate_auth_cookie;
use crate::{error::AuthAPIError, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// This function handles the login request.
/// It validates the email and password, checks if the user exists,
/// and if 2FA is required.
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

    let user_store = state.user_store.read().await;

    let user = user_store
        .get_user(&email)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;
    if user.requires_2fa {
        handle_2fa(&email, &state, jar).await
    } else {
        handle_no_2fa(&email, jar).await
    }
}

/// This function handles the case where 2FA is not required for login.
/// It generates a new auth cookie and returns it in the response.
async fn handle_no_2fa(
    email: &Email,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let auth_cookie = generate_auth_cookie(&email).map_err(|_| AuthAPIError::UnexpectedError)?;
    let updated_jar = jar.add(auth_cookie);
    Ok((updated_jar, (StatusCode::OK, Json(LoginResponse::No2FA))))
}

/// This function handles the case where 2FA is required for login.
/// It generates a new 2FA code and stores it in the 2FA code store.
async fn handle_2fa(
    email: &Email,
    state: &AppState,
    jar: CookieJar,
) -> Result<(CookieJar, (StatusCode, Json<LoginResponse>)), AuthAPIError> {
    let two_fa_code = TwoFACode::new();
    let login_attempt_id = LoginAttemptId::new();

    // Store the 2FA code and the login attempt ID in the 2FA code store
    state
        .two_fa_code_store
        .write()
        .await
        .add_code(email, two_fa_code.clone(), login_attempt_id.clone())
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    // Send the 2FA code to the user via email
    let (subject, content) =
        crate::domain::email_client::two_fa_login_email_template(email, two_fa_code.as_ref());

    state
        .email_client
        .read()
        .await
        .send_email(email, &subject, &content)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((
        jar,
        (
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::With2FA(TwoFactorLoginResponse {
                message: "2FA required".to_string(),
                login_attempt_id: login_attempt_id.as_ref().to_string(),
            })),
        ),
    ))
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    No2FA,
    With2FA(TwoFactorLoginResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorLoginResponse {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}
