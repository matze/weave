use std::convert::Infallible;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::Key;

use crate::Issuer;

/// Extract authentication status based on the presence and validity of an issued JSON web token in
/// a request cookie. The inner bool says if authentication is valid or not.
#[derive(Debug)]
pub(crate) struct Authenticated(pub bool);

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    Key: FromRef<S>,
    Issuer: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state).await;
        let issuer = Issuer::from_ref(state);

        let authenticated = jar
            .map(|jar| {
                jar.get("jwt")
                    .and_then(|cookie| Some(Authenticated(issuer.is_valid(cookie.value_trimmed()))))
            })
            .ok()
            .flatten()
            .unwrap_or(Authenticated(false));

        Ok(authenticated)
    }
}
