use maud::{DOCTYPE, Markup, html};

use crate::partials;
use crate::{Notebook, assets};

/// Render the main page layout.
///
/// When `show_note` is true, the sidebar is hidden and the note content is
/// visible on mobile. This is used when rendering `/note/` pages directly.
pub(crate) fn layout(
    authenticated: bool,
    notebook: Notebook,
    content: Markup,
    show_note: bool,
) -> Markup {
    let notebook = notebook.lock().unwrap();
    let notes = notebook.all_notes((!authenticated).then_some("public"));

    let icon = if authenticated {
        assets::icons::sign_out()
    } else {
        assets::icons::sign_in()
    };

    html! {
        (DOCTYPE)
        html lang="en" {
            (partials::head::head())

            body class="font-sans bg-gray-100 dark:bg-gray-900 text-black dark:text-white" {
              div class="max-w-7xl mx-auto flex flex-col md:flex-row h-screen bg-white dark:bg-gray-800" {
                div id="sidebar" class={"w-full md:w-80 border-b md:border-b-0 md:border-r border-gray-200 dark:border-gray-700 flex flex-col overflow-y-auto flex-shrink-0 h-screen md:h-auto" @if show_note { " mobile-hidden" }} {
                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                        div class="flex" {
                            div class="flex flex-col flex-auto" {
                                span class="text-lg font-bold text-transparent bg-clip-text bg-gradient-to-r from-sky-600 to-green-600" {"weave"}
                            }

                            div class="flex flex-row flex-none items-center gap-2 mr-2" {
                                a href="/login" {
                                    (icon)
                                }
                            }
                        }
                    }

                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                        div class="relative" {
                            input #filter-input type="search"
                                name="query"
                                placeholder="Filter notes..."
                                class="w-full p-2 pr-8 rounded bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                hx-post="/f/search"
                                hx-trigger="input changed delay:300ms, keyup[key=='Enter'], notes-updated from:body"
                                hx-target="#search-list"
                                hx-swap="innerHTML"
                                {}

                            button #filter-clear type="button"
                                class="absolute cursor-pointer right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hidden"
                                onclick="let i=document.getElementById('filter-input');i.value='';htmx.ajax('POST','/f/search',{target:'#search-list',values:{query:''}});this.classList.add('hidden')" {
                                    (assets::icons::cancel())
                                }
                        }
                        script {
                            (maud::PreEscaped("document.getElementById('filter-input').addEventListener('input',function(){document.getElementById('filter-clear').classList.toggle('hidden',!this.value)})"))
                        }
                    }

                    div class="flex-grow overflow-y-auto max-h-xs md:max-h-none" id="search-list" {
                        (partials::note_list::note_list(notes))
                    }
                }

                div class={"flex flex-grow flex-col overflow-y-auto h-screen md:h-auto md:basis-1/2" @if show_note { " mobile-visible" }} id="note-content" {
                    (content)
                }
              }

                script src="/htmx.2.0.4.min.js" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" {}
                script src="/app.js" {}
            }
        }
    }
}
