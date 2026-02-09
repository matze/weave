use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::zk::Note;
use crate::{Notebook, assets, md, partials};

pub(crate) async fn note(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    let (notes, note_data): (Vec<Note>, Option<(String, String, bool)>) = {
        let notebook = notebook.lock().unwrap();
        let tag_filter = (!authenticated).then_some("public");
        let notes: Vec<Note> = notebook
            .all_notes(tag_filter)
            .into_iter()
            .cloned()
            .collect();
        let note_data = notebook
            .note(&stem)
            .map(|n| (n.title.clone(), n.body.clone(), n.has("public")));
        (notes, note_data)
    };

    // Render note content
    let note_content = if let Some((title, body, is_public)) = note_data {
        if !authenticated && !is_public {
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
                        h2 class="text-xl font-bold dark:text-white" { (title) }
                        @if authenticated {
                            button
                                class="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-500 dark:text-gray-400"
                                hx-get={ "/f/" (stem) "/edit" }
                                hx-target="#note-content"
                                aria-label="Edit note" {
                                (assets::icons::pencil())
                            }
                        }
                    }
                }

                div class="flex-grow px-4 pt-6 pb-4 overflow-y-auto" {
                    div class="prose max-w-none dark:prose-invert" {
                        (tokio::task::spawn_blocking(move || md::markdown_to_html(&body))
                            .await
                            .expect("join working"))
                    }
                }
            }
        }
    } else {
        html! {
            div class="h-screen flex items-center justify-center" {
                div class="flex flex-col items-center justify-center p-8" {
                    h2 class="text-xl font-bold" { "note not found" }
                }
            }
        }
    };

    partials::layout::layout(authenticated, &notes, Some(note_content))
}
