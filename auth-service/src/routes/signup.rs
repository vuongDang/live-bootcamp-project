use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Deserialize;
use crate::{domain::user::User, error::AuthAPIError, AppState};
use crate::domain::data_stores::UserStoreError;

pub async fn signup(State(state): State<AppState>, Json(request): Json<SignupRequest>) -> Result<impl IntoResponse, AuthAPIError> {
    let user = User::new(request.email, request.password, request.requires_2fa);
    if user.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let res = state.user_store.write().await.add_user(user.unwrap()).await;
    match res {
        Ok(_) => Ok((StatusCode::CREATED, Json("User created successfully"))),
        Err(err) => Err(match err {
            UserStoreError::UserAlreadyExists => AuthAPIError::UserAlreadyExists,
            UserStoreError::UnexpectedError => AuthAPIError::UnexpectedError,
            _ => unreachable!("Unexpected error: {:?}", err),
        }),
    }
}


#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}