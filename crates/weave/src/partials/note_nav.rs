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
        nav class="note-nav" {
            @if !data.headings.is_empty() {
                section class="note-nav-section" {
                    h4 class="note-nav-title" { "On this page" }
                    ul class="note-nav-list note-nav-toc" {
                        @for h in &data.headings { (toc_item(h, min_level)) }
                    }
                }
            }

            @if !data.outgoing_links.is_empty() {
                section class="note-nav-section" {
                    h4 class="note-nav-title" { "Linking to" }
                    ul class="note-nav-list" {
                        @for note in &data.outgoing_links {
                            li {
                                a href="#"
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
                section class="note-nav-section" {
                    h4 class="note-nav-title" { "Linked from" }
                    ul class="note-nav-list" {
                        @for note in &data.backlinks {
                            li {
                                a href="#"
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
                section class="note-nav-section" {
                    h4 class="note-nav-title" { "Tags" }
                    div class="tag-chips" {
                        @for tag in &data.tags {
                            a href="#" class="tag-chip"
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
    html! {
        li data-depth=(depth) {
            a href={ "#" (h.anchor) }
                onclick="gotoHeading(event, this.getAttribute('href').slice(1))"
            { (h.text) }
        }
    }
}
