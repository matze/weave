use axum::extract::{Path, State};
use axum::http::StatusCode;
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::partials::note_nav::{NoteNavData, note_nav};
use crate::{Notebook, md};

const HX_TRIGGER: axum::http::HeaderName = axum::http::HeaderName::from_static("hx-trigger");

fn edit_form(stem: &str, body: &str) -> Markup {
    html! {
        article class="note" data-stem=(stem) data-mode="edit" {
            form class="editor" {
                textarea id="editor-textarea" name="body" class="editor-input"
                    placeholder="Write your note here…" {
                    (body)
                }
                footer class="editor-actions" {
                    button type="button" id="editor-cancel" class="btn btn-ghost"
                        hx-get={ "/f/" (stem) }
                        hx-target="#note-content"
                        hx-push-url="false" {
                        "Cancel"
                    }
                    button type="button" id="editor-save" class="btn btn-primary"
                        hx-put={ "/f/" (stem) }
                        hx-include="#editor-textarea"
                        hx-target="#note-content" {
                        "Save"
                    }
                }
            }
        }
    }
}

pub(crate) async fn edit(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Result<Markup, StatusCode> {
    if !authenticated {
        return Err(StatusCode::FORBIDDEN);
    }

    let content = notebook
        .lock()
        .unwrap()
        .note(&stem)
        .ok_or(StatusCode::NOT_FOUND)?
        .raw_content()
        .to_owned();

    Ok(edit_form(&stem, &content))
}

pub(crate) async fn save(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
    axum::extract::Form(Body { body }): axum::extract::Form<Body>,
) -> Result<([(axum::http::HeaderName, &'static str); 1], Markup), StatusCode> {
    if !authenticated {
        return Err(StatusCode::FORBIDDEN);
    }

    let stem_clone = stem.clone();

    let (title, rendered, nav_data) = tokio::task::spawn_blocking(move || {
        let mut notebook = notebook.lock().unwrap();

        let file_path = notebook
            .note(&stem_clone)
            .ok_or(StatusCode::NOT_FOUND)?
            .abs_path()
            .to_owned();

        std::fs::write(&file_path, &body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        notebook
            .reload(&stem_clone)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let note = notebook.note(&stem_clone).ok_or(StatusCode::NOT_FOUND)?;
        let backlinks = notebook.backlinks(&stem_clone, true);
        let outgoing_links = notebook.outgoing_links(note.outgoing_links(), true);
        let tags = note.tags().to_vec();
        let title = note.title().to_owned();
        let body = note.body().to_owned();

        let (rendered, headings) = md::markdown_to_html_with_headings(&body);

        let nav_data = NoteNavData {
            headings,
            outgoing_links,
            backlinks,
            tags,
        };

        Ok::<_, StatusCode>((title, rendered, nav_data))
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    let has_rail = !nav_data.is_empty();
    let body_class = if has_rail {
        "note-body"
    } else {
        "note-body no-rail"
    };

    Ok((
        [(HX_TRIGGER, "notes-updated")],
        html! {
            article class="note" data-stem=(stem) data-mode="read" {
                header class="note-head" { h1 { (title) } }
                div class=(body_class) {
                    div class="md" { (rendered) }
                    @if has_rail { (note_nav(&nav_data)) }
                }
            }
        },
    ))
}

pub(crate) async fn preview(
    Authenticated(authenticated): Authenticated,
    form: axum::extract::Form<Body>,
) -> Result<Markup, StatusCode> {
    if !authenticated {
        return Err(StatusCode::FORBIDDEN);
    }

    let body = form.0.body;
    let rendered = tokio::task::spawn_blocking(move || md::markdown_to_html(&body))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rendered)
}

#[derive(serde::Deserialize)]
pub(crate) struct Body {
    body: String,
}
