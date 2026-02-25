use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::partials::note_nav::{NoteNavData, note_nav};
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
        return html! {
            div class="p-4 border-b border-gray-200 dark:border-gray-700 flex-shrink-0" {
                div class="flex items-center text-xl" {
                    span class="invisible font-black" { "\u{00a0}" }
                    div class="ml-auto" {
                        a href="/login" aria-label="Sign in" {
                            (assets::icons::sign_in())
                        }
                    }
                }
            }
            div class="flex-grow flex items-center justify-center" {
                div class="flex flex-col items-center justify-center p-8" {
                    h2 class="text-xl font-bold" { "access denied" }
                }
            }
        };
    }

    let backlinks = notebook.lock().unwrap().backlinks(&stem, authenticated);
    let outgoing_links = notebook.lock().unwrap().outgoing_links(note.outgoing_links(), authenticated);
    let tags = note.tags().to_vec();
    let body = note.body().to_owned();
    let title = note.title().to_owned();

    let (rendered, headings) = tokio::task::spawn_blocking(move || {
        md::markdown_to_html_with_headings(&body)
    })
    .await
    .expect("join working");

    let nav_data = NoteNavData {
        headings,
        outgoing_links,
        backlinks,
        tags,
    };

    html! {
        div class="p-4 border-b border-gray-200 dark:border-gray-700 flex-shrink-0" {
            div class="flex items-center gap-3" {
                button
                    class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
                    onclick="goBack()"
                    aria-label="Back to notes" {
                    (assets::icons::back())
                }
                h2 class="text-xl font-bold dark:text-white flex-grow" { (title) }
                div class="flex items-center gap-4 ml-auto" {
                    @if authenticated {
                        button
                            class="cursor-pointer"
                            hx-get={ "/f/" (stem) "/edit" }
                            hx-target="#note-content"
                            aria-label="Edit note" {
                            (assets::icons::pencil())
                        }
                    }
                    @if authenticated {
                        a href="/logout" aria-label="Sign out" {
                            (assets::icons::sign_out())
                        }
                    } @else {
                        a href="/login" aria-label="Sign in" {
                            (assets::icons::sign_in())
                        }
                    }
                }
            }
        }

        div class="flex flex-row flex-grow overflow-hidden min-h-0" {
            div class="flex-grow px-4 pt-6 pb-4 overflow-y-auto min-w-0" {
                div class="prose dark:prose-invert" { (rendered) }
            }
            (note_nav(&nav_data))
        }
    }
}
