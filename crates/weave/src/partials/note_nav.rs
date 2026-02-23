use maud::{Markup, html};

use crate::md::Heading;
use crate::zk::Note;

pub(crate) struct NoteNavData {
    pub headings: Vec<Heading>,
    pub outgoing_links: Vec<Note>,
    pub backlinks: Vec<Note>,
    pub tags: Vec<String>,
}

impl NoteNavData {
    pub fn is_empty(&self) -> bool {
        self.headings.is_empty()
            && self.outgoing_links.is_empty()
            && self.backlinks.is_empty()
            && self.tags.is_empty()
    }
}

pub(crate) fn note_nav(data: &NoteNavData) -> Markup {
    if data.is_empty() {
        return html! {};
    }

    let min_level = data.headings.iter().map(|h| h.level).min().unwrap_or(1);

    html! {
        nav class="hidden lg:flex flex-col w-56 flex-shrink-0 border-l border-gray-200 dark:border-gray-700 overflow-y-auto text-sm pt-4" {

            @if !data.headings.is_empty() {
                div class="px-4 py-2" {
                    p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2" {
                        "On this page"
                    }
                    ul { @for h in &data.headings { (toc_item(h, min_level)) } }
                }
            }

            @if !data.outgoing_links.is_empty() {
                div class="px-4 py-2" {
                    p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2" {
                        "Linking to"
                    }
                    ul class="space-y-1" {
                        @for note in &data.outgoing_links {
                            li {
                                a href="#"
                                    class="text-sky-600 hover:underline"
                                    hx-get={ "/f/" (note.filename_stem()) }
                                    hx-target="#note-content"
                                    hx-push-url={ "/note/" (note.filename_stem()) }
                                { (note.title()) }
                            }
                        }
                    }
                }
            }

            @if !data.backlinks.is_empty() {
                div class="px-4 py-2" {
                    p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2" {
                        "Linked from"
                    }
                    ul class="space-y-1" {
                        @for note in &data.backlinks {
                            li {
                                a href="#"
                                    class="text-sky-600 hover:underline"
                                    hx-get={ "/f/" (note.filename_stem()) }
                                    hx-target="#note-content"
                                    hx-push-url={ "/note/" (note.filename_stem()) }
                                { (note.title()) }
                            }
                        }
                    }
                }
            }

            @if !data.tags.is_empty() {
                div class="px-4 py-2" {
                    p class="text-xs font-semibold uppercase tracking-wide text-gray-500 dark:text-gray-400 mb-2" {
                        "Tags"
                    }
                    div class="flex flex-wrap gap-1" {
                        @for tag in &data.tags {
                            a href="#"
                                class="inline-block px-2 py-0.5 rounded bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-600 text-xs"
                                hx-post="/f/search"
                                hx-vals={ "{\"query\": \"#" (tag) "\"}" }
                                hx-target="#search-list"
                                onclick="showList()"
                            { "#" (tag) }
                        }
                    }
                }
            }
        }
    }
}

fn toc_item(h: &Heading, min_level: u8) -> Markup {
    let depth = h.level.saturating_sub(min_level);
    let indent = match depth {
        0 => "pl-0",
        1 => "pl-3",
        2 => "pl-6",
        3 => "pl-9",
        _ => "pl-12",
    };
    html! {
        li class={ (indent) " py-0.5" } {
            a href={ "#" (h.anchor) }
                class="text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 hover:underline truncate block"
            { (h.text) }
        }
    }
}
