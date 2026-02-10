use axum::Form;
use axum::extract::State;
use maud::Markup;
use serde::Deserialize;

use crate::extract::Authenticated;
use crate::{Notebook, partials};

#[derive(Deserialize, Debug)]
pub(crate) struct Search {
    query: String,
}

/// Return fragment for the sidebar search results (filters notes list).
#[tracing::instrument(skip(notebook))]
pub(crate) async fn search(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Form(search): Form<Search>,
) -> Markup {
    let query = search.query.trim();
    let notebook = notebook.lock().unwrap();
    let tag_filter = (!authenticated).then_some("public");

    let notes = if query.is_empty() {
        // Return all authorized notes when query is empty
        notebook.all_notes(tag_filter)
    } else if let Some(tag) = query.strip_prefix('#') {
        if !authenticated {
            notebook.search_tag("public")
        } else {
            notebook.search_tag(tag)
        }
    } else {
        notebook.search_titles(query, tag_filter)
    };

    tracing::info!(number = notes.len(), "search results");

    partials::note_list::note_list(notes)
}
