use axum::extract::State;
use maud::{DOCTYPE, Markup, html};

use crate::extract::Authenticated;
use crate::{Notebook, assets, partials};

pub(crate) async fn index(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
) -> Markup {
    let icon = if authenticated {
        assets::icons::unlocked()
    } else {
        assets::icons::locked()
    };

    // Hold the lock during rendering to avoid cloning notes
    let notebook = notebook.lock().unwrap();
    let tag_filter = (!authenticated).then_some("public");
    let notes = notebook.all_notes(tag_filter);

    html! {
        (DOCTYPE)
        html lang="en" {
            head { (partials::head::head()) }
            body class="flex flex-col md:flex-row h-screen bg-white dark:bg-gray-800 text-black dark:text-white" {
                div id="sidebar" class="w-full md:w-80 border-b md:border-b-0 md:border-r border-gray-200 dark:border-gray-700 flex flex-col overflow-y-auto flex-shrink-0 h-screen md:h-auto" {
                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                        div class="flex" {
                            div class="flex flex-col flex-auto" {
                                span class="text-lg font-bold text-transparent bg-clip-text bg-gradient-to-r from-sky-600 to-green-600" {"weave"}
                            }

                            div class="flex flex-col flex-none" {
                                a href="/login" {
                                    (icon)
                                }
                            }
                        }
                    }

                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                        input type="search"
                            name="query"
                            placeholder="Filter notes..."
                            class="w-full p-2 rounded bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            hx-post="/f/search"
                            hx-trigger="input changed delay:300ms, keyup[key=='Enter']"
                            hx-target="#search-list"
                            hx-swap="innerHTML"
                            {}
                    }

                    div class="flex-grow overflow-y-auto max-h-xs md:max-h-none" id="search-list" {
                        (partials::note_list::note_list(notes))
                    }
                }

                div class="flex flex-grow flex-col overflow-y-auto h-screen md:h-auto md:basis-1/2" id="note-content" {}

                // HTMX script
                script src="/htmx.2.0.4.min.js" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" {}
            }
        }
    }
}
