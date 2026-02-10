use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::{Notebook, assets, md};

/// Return note content fragment consisting of title div and rendered markdown.
pub(crate) async fn note(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    let Some(note) = notebook.lock().unwrap().note(&stem) else {
        return html! {};
    };

    if !authenticated && !note.has("public") {
        html! {
            div class="h-screen flex items-center justify-center" {
                div class="flex flex-col items-center justify-center p-8" {
                    h2 class="text-xl font-bold" { "access denied" }
                }
            }
        }
    } else {
        html! {
            div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                div class="flex items-center gap-3" {
                    button
                        class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                        onclick="showSidebar()"
                        aria-label="Back to notes" {
                        (assets::icons::back())
                    }
                    h2 class="text-xl font-bold dark:text-white" { (note.title) }
                    @if authenticated {
                        button
                            class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400 cursor-pointer"
                            hx-get={ "/f/" (stem) "/edit" }
                            hx-target="#note-content"
                            aria-label="Edit note" {
                            (assets::icons::pencil())
                        }
                    }
                }
            }

            div class="flex-grow px-4 pt-6 pb-4 overflow-y-auto" {
                div class="prose dark:prose-invert" {
                    (tokio::task::spawn_blocking(move || md::markdown_to_html(&note.body))
                        .await
                        .expect("join working"))
                }
            }
        }
    }
}
