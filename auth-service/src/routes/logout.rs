use crate::{
    error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
    AppState,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
    let jwt = jar
        .get(JWT_COOKIE_NAME)
        .map_or_else(
            || Err(AuthAPIError::MissingToken),
            |cookie| Ok(cookie.value()),
        )?
        .to_owned();
    validate_token(&jwt)
        .await
        .map_err(|_| AuthAPIError::InvalidToken)?;

    // Invalidate the JWT by removing it from the cookie jar
    let jar = jar.remove(JWT_COOKIE_NAME);

    state
        .banned_token_store
        .write()
        .await
        .add_banned_token(&jwt)
        .await
        .map_err(|_| AuthAPIError::UnexpectedError)?;

    Ok((jar, StatusCode::OK.into_response()))
}
