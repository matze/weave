use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::partials::note_nav::{NoteNavData, note_nav};
use crate::{Notebook, md};

/// Return note content fragment: <article class="note"> with header + body.
pub(crate) async fn note(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    let Some(note) = notebook.lock().unwrap().note(&stem) else {
        return html! {};
    };

    if !authenticated && !note.has("public") {
        return html! {
            article class="note" data-stem=(stem) data-mode="read" {
                div class="note-empty" { "access denied" }
            }
        };
    }

    let backlinks = notebook.lock().unwrap().backlinks(&stem, authenticated);
    let outgoing_links = notebook
        .lock()
        .unwrap()
        .outgoing_links(note.outgoing_links(), authenticated);
    let tags = note.tags().to_vec();
    let body = note.body().to_owned();
    let title = note.title().to_owned();

    let (rendered, headings) =
        tokio::task::spawn_blocking(move || md::markdown_to_html_with_headings(&body))
            .await
            .expect("join working");

    let nav_data = NoteNavData {
        headings,
        outgoing_links,
        backlinks,
        tags,
    };
    let has_rail = !nav_data.is_empty();

    let body_class = if has_rail {
        "note-body"
    } else {
        "note-body no-rail"
    };
    let note_class = if has_rail { "note" } else { "note note--no-rail" };

    html! {
        article class=(note_class) data-stem=(stem) data-mode="read" {
            header class="note-head" { h1 { (title) } }
            div class=(body_class) {
                div class="md" { (rendered) }
                @if has_rail { (note_nav(&nav_data)) }
            }
        }
    }
}
