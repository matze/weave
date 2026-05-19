use axum::extract::State;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::IntoResponse;

use crate::Notebook;
use crate::extract::Authenticated;

const HX_REDIRECT: HeaderName = HeaderName::from_static("hx-redirect");

/// Create an empty note and tell HTMX to navigate to its `/note/{stem}` page.
pub(crate) async fn create(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
) -> Result<impl IntoResponse, StatusCode> {
    if !authenticated {
        return Err(StatusCode::FORBIDDEN);
    }

    let stem = notebook
        .lock()
        .unwrap()
        .create_note("# Untitled\n")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let location = format!("/note/{stem}");
    let value = HeaderValue::from_str(&location).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(([(HX_REDIRECT, value)], StatusCode::NO_CONTENT))
}
