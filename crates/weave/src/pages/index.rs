use axum::extract::State;
use maud::Markup;

use crate::extract::Authenticated;
use crate::{Notebook, partials};

pub(crate) async fn index(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
) -> Markup {
    let notebook = notebook.lock().unwrap();
    let tag_filter = (!authenticated).then_some("public");
    let notes = notebook.all_notes(tag_filter);

    partials::layout::layout(authenticated, notes, None, None)
}
