use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use crate::{domain::user::User, error::AuthAPIError, services::hashmap_user_store::UserStoreError, AppState};

pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let user = User::new(request.email, request.password, request.requires_2fa);
    if !is_valid_email(&user.email) || !is_valid_password(&user.password) {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let res = state.user_store.write().await.add_user(user);
    match res {
        Ok(_) => Ok((StatusCode::CREATED, Json("User created successfully"))),
        Err(err) => Err(match err {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            UserStoreError::UnexpectedError => AuthAPIError::UnexpectedError,
            _ => unreachable!("Unexpected error: {:?}", err),
        }),
    }
}

// Simple regex for email validation
fn is_valid_email(email: &str) -> bool {
    let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").expect("Failed to compile regex");
    re.is_match(email)
}

// Password must be at least 8 characters long and contain at least one digit
fn is_valid_password(password: &str) -> bool {
    password.len() >= 8 && password.chars().any(|c| c.is_digit(10))
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}