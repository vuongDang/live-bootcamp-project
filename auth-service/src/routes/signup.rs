use crate::domain::data_stores::UserStoreError;
use crate::{domain::user::User, error::AuthAPIError, AppState};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use secrecy::Secret;
use serde::Deserialize;

#[tracing::instrument(name = "Signup", skip_all)]
pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    let user = User::new(request.email, request.password, request.requires_2fa);
    if user.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let res = state.user_store.write().await.add_user(user.unwrap()).await;
    match res {
        Ok(_) => Ok((StatusCode::CREATED, Json("User created successfully"))),
        Err(err) => Err(match err {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            UserStoreError::UnexpectedError(e) => AuthAPIError::UnexpectedError(e.into()),
            _ => unreachable!("Unexpected error: {:?}", err),
        }),
    }
}

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: Secret<String>,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}
