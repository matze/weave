use maud::{DOCTYPE, Markup, html};

use crate::partials;
use crate::{Notebook, assets};

/// Render the main page layout.
///
/// When `show_note` is true the body carries `data-note`, which hides the
/// sidebar on mobile. The shell uses a CSS grid so that focus mode can slide
/// the chrome out without reflowing the note column.
pub(crate) fn layout(
    authenticated: bool,
    notebook: Notebook,
    content: Markup,
    show_note: bool,
) -> Markup {
    let notebook = notebook.lock().unwrap();
    let notes = notebook.all_notes((!authenticated).then_some("public"));

    html! {
        (DOCTYPE)
        html lang="en" {
            (partials::head::head())

            body data-note?[show_note] {
              div class="shell" {
                header class="topbar" {
                    a class="brand" href="/" { "weave" }

                    div class="topbar-mid" {
                        div class="search" {
                            span class="search-icon" { (assets::icons::search()) }
                            input #filter-input type="search"
                                name="query"
                                placeholder="Search notes by title, tag, body..."
                                autocomplete="off"
                                hx-post="/f/search"
                                hx-trigger="input changed delay:300ms, keyup[key=='Enter'], notes-updated from:body"
                                hx-target="#search-list"
                                hx-swap="innerHTML"
                                {}
                            span class="search-kbd" { "/" }
                        }
                    }

                    div class="actions" {
                        div class="segctl" #mode-segctl {
                            button type="button" #mode-read class="is-on" data-mode="read"
                                title="Read (V)" aria-label="Read" {
                                (assets::icons::eye()) span { "Read" }
                            }
                            button type="button" #mode-edit data-mode="edit"
                                title="Edit (E)" aria-label="Edit" {
                                (assets::icons::pencil()) span { "Edit" }
                            }
                        }
                        @if authenticated {
                            button type="button" class="tb-btn" #clip-toggle
                                title="Clip URL (C)" aria-label="Clip URL" {
                                (assets::icons::link())
                            }
                            button type="button" class="tb-btn" #new-note
                                title="New note (N)" aria-label="New note"
                                hx-post="/note"
                                hx-swap="none" {
                                (assets::icons::plus())
                            }
                        }
                        button type="button" class="tb-btn" #theme-toggle
                            title="Toggle theme (D)" aria-label="Toggle theme" {
                            (assets::icons::moon())
                        }
                        @if authenticated {
                            a href="/logout" class="tb-btn" aria-label="Sign out" title="Sign out" {
                                (assets::icons::sign_out())
                            }
                        } @else {
                            a href="/login" class="tb-btn" aria-label="Sign in" title="Sign in" {
                                (assets::icons::sign_in())
                            }
                        }
                    }
                }

                @if authenticated {
                    div class="clip-drawer" #clip-drawer {
                        h3 { "Clip a URL" }
                        div class="clip-row" {
                            input #clip-input type="url" name="url"
                                placeholder="https://…"
                                hx-post="/clip"
                                hx-trigger="keyup[key=='Enter']"
                                hx-swap="none"
                                {}
                            button type="button" class="btn btn-ghost" #clip-cancel {
                                "Cancel"
                            }
                        }
                    }
                }

                div class="body-grid" {
                    aside class="sidebar" {
                        div id="search-list" class="note-list" {
                            (partials::note_list::note_list(notes))
                        }
                    }

                    main class="main" id="note-content" {
                        @if content.0.is_empty() {
                            (welcome(authenticated))
                        } @else {
                            (content)
                        }
                    }
                }

              }

                script src="/htmx.2.0.4.min.js" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" {}
                script src="/app.js" {}
            }
        }
    }
}

fn welcome(authenticated: bool) -> Markup {
    html! {
        div class="welcome" {
            p class="welcome-tip" {
                "Press "
                span class="kbd" { "/" }
                " to search, "
                span class="kbd" { "J" }
                "/"
                span class="kbd" { "K" }
                " to navigate"
                @if authenticated {
                    ", "
                    span class="kbd" { "N" }
                    " for a new note, "
                    span class="kbd" { "C" }
                    " to clip a URL"
                }
                ", "
                span class="kbd" { "D" }
                " to toggle theme."
            }
        }
    }
}
