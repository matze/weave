use axum::extract::{Path, State};
use axum::http::StatusCode;
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::{Notebook, assets, md};

const HX_TRIGGER: axum::http::HeaderName = axum::http::HeaderName::from_static("hx-trigger");

fn edit_form(stem: &str, body: &str) -> Markup {
    html! {
        div class="flex flex-col h-full" {
            div class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700 flex-shrink-0" {
                button
                    class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    onclick="goBack()"
                    aria-label="Back to notes" {
                    (assets::icons::back())
                }
                div class="flex-grow" {}
                button id="editor-save"
                    class="px-4 py-1.5 cursor-pointer font-medium text-white bg-sky-600 rounded hover:bg-sky-700"
                    hx-put={ "/f/" (stem) }
                    hx-include="#editor-textarea"
                    hx-target="#note-content" {
                    "Save"
                }
                button
                    class="px-4 py-1.5 cursor-pointer font-medium text-gray-600 dark:text-gray-300 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    hx-get={ "/f/" (stem) }
                    hx-target="#note-content"
                    hx-push-url="false" {
                    "Cancel"
                }
            }

            textarea id="editor-textarea"
                name="body"
                class="flex-grow p-4 font-mono bg-white dark:bg-gray-800 text-black dark:text-white resize-none focus:outline-none"
                placeholder="Write your note here..." {
                (body)
            }

            div id="preview-area" class="hidden flex-grow px-4 pt-6 pb-4 overflow-y-auto" {
                div id="preview-content" class="prose dark:prose-invert" {}
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

    let note = tokio::task::spawn_blocking(move || {
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

        notebook.note(&stem_clone).ok_or(StatusCode::NOT_FOUND)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)??;

    Ok((
        [(HX_TRIGGER, "notes-updated")],
        html! {
            div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                div class="flex items-center gap-3" {
                    button
                        class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                        onclick="goBack()"
                        aria-label="Back to notes" {
                        (assets::icons::back())
                    }
                    h2 class="text-xl font-bold dark:text-white" { (note.title()) }
                    button
                        class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400"
                        hx-get={ "/f/" (stem) "/edit" }
                        hx-target="#note-content"
                        aria-label="Edit note" {
                        (assets::icons::pencil())
                    }
                }
            }

            div class="flex-grow px-4 pt-6 pb-4 overflow-y-auto" {
                div class="prose dark:prose-invert" {
                    (tokio::task::spawn_blocking(move || md::markdown_to_html(note.body()))
                        .await
                        .expect("join working"))
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
