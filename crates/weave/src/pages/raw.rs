use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::Notebook;
use crate::extract::Authenticated;

/// Serve the raw on-disk Markdown (frontmatter + body) of a note as `text/markdown`.
///
/// Mirrors the access check used for the rendered fragment: unauthenticated
/// requests only succeed for notes tagged `public`.
pub(crate) async fn raw(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Response {
    let Some(note) = notebook.lock().unwrap().note(&stem) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if !authenticated && !note.has("public") {
        return StatusCode::FORBIDDEN.into_response();
    }

    (
        [(header::CONTENT_TYPE, "text/markdown; charset=utf-8")],
        note.raw_content().to_owned(),
    )
        .into_response()
}
