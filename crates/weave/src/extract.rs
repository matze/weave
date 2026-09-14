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

/// Whether login is disabled (no password configured). When true, every request counts as
/// authenticated.
#[derive(Debug, Clone, Copy)]
pub(crate) struct LoginDisabled(pub bool);

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    Key: FromRef<S>,
    Issuer: FromRef<S>,
    LoginDisabled: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if LoginDisabled::from_ref(state).0 {
            return Ok(Authenticated(true));
        }

        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state).await;
        let issuer = Issuer::from_ref(state);

        let authenticated = jar
            .map(|jar| {
                jar.get("jwt")
                    .map(|cookie| Authenticated(issuer.is_valid(cookie.value_trimmed())))
            })
            .ok()
            .flatten()
            .unwrap_or(Authenticated(false));

        Ok(authenticated)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::{Request, header};
    use axum::response::IntoResponse;
    use axum_extra::extract::cookie::Cookie;

    use super::*;

    #[derive(Clone)]
    struct TestState {
        key: Key,
        issuer: Issuer,
        login_disabled: bool,
    }

    impl TestState {
        fn new(login_disabled: bool) -> Self {
            Self {
                key: Key::generate(),
                issuer: Arc::new(crate::jwt::Issuer::new().unwrap()),
                login_disabled,
            }
        }

        /// A `Cookie` header value carrying `jwt=<token>` signed with this state's key.
        fn signed_cookie(&self, token: String) -> String {
            let jar = SignedCookieJar::new(self.key.clone()).add(Cookie::new("jwt", token));
            let response = jar.into_response();
            let set_cookie = response.headers().get(header::SET_COOKIE).unwrap();
            let set_cookie = set_cookie.to_str().unwrap();
            set_cookie.split(';').next().unwrap().to_owned()
        }
    }

    impl FromRef<TestState> for Key {
        fn from_ref(state: &TestState) -> Self {
            state.key.clone()
        }
    }

    impl FromRef<TestState> for Issuer {
        fn from_ref(state: &TestState) -> Self {
            state.issuer.clone()
        }
    }

    impl FromRef<TestState> for LoginDisabled {
        fn from_ref(state: &TestState) -> Self {
            LoginDisabled(state.login_disabled)
        }
    }

    async fn authenticated(state: &TestState, cookie: Option<&str>) -> bool {
        let mut request = Request::builder().uri("/");

        if let Some(cookie) = cookie {
            request = request.header(header::COOKIE, cookie);
        }

        let (mut parts, ()) = request.body(()).unwrap().into_parts();
        let Ok(Authenticated(authenticated)) =
            Authenticated::from_request_parts(&mut parts, state).await;
        authenticated
    }

    #[tokio::test]
    async fn login_disabled_grants_access_without_cookie() {
        let state = TestState::new(true);
        assert!(authenticated(&state, None).await);
    }

    #[tokio::test]
    async fn login_disabled_grants_access_with_bogus_cookie() {
        let state = TestState::new(true);
        assert!(authenticated(&state, Some("jwt=not-a-token")).await);
    }

    #[tokio::test]
    async fn login_enabled_denies_access_without_cookie() {
        let state = TestState::new(false);
        assert!(!authenticated(&state, None).await);
    }

    #[tokio::test]
    async fn login_enabled_grants_access_with_valid_token() {
        let state = TestState::new(false);
        let cookie = state.signed_cookie(state.issuer.new_token());
        assert!(authenticated(&state, Some(&cookie)).await);
    }

    #[tokio::test]
    async fn login_enabled_denies_unsigned_cookie() {
        let state = TestState::new(false);
        let cookie = format!("jwt={}", state.issuer.new_token());
        assert!(!authenticated(&state, Some(&cookie)).await);
    }

    #[tokio::test]
    async fn login_enabled_denies_token_from_other_issuer() {
        let state = TestState::new(false);
        let other = crate::jwt::Issuer::new().unwrap();
        let cookie = state.signed_cookie(other.new_token());
        assert!(!authenticated(&state, Some(&cookie)).await);
    }
}
