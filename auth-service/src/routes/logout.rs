use crate::{
    error::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};
use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::CookieJar;

pub async fn logout(jar: CookieJar) -> Result<(CookieJar, impl IntoResponse), AuthAPIError> {
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

    let jar = jar.remove(JWT_COOKIE_NAME);
    Ok((jar, StatusCode::OK.into_response()))
}
