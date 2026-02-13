use maud::{Markup, html};

use crate::assets::icons;
use crate::zk::{self, NoteExt};

enum Note<'a> {
    Regular(&'a zk::Note),
    Pinned(&'a zk::Note),
    Archived(&'a zk::Note),
}

impl<'a> Note<'a> {
    fn new(note: &'a zk::Note) -> Self {
        if note.has("pin") {
            Self::Pinned(note)
        } else if note.has("archived") {
            Self::Archived(note)
        } else {
            Self::Regular(note)
        }
    }

    fn inner(&self) -> &zk::Note {
        match self {
            Self::Regular(n) | Self::Pinned(n) | Self::Archived(n) => n,
        }
    }
}

/// Render a list of notes for the sidebar.
///
/// Notes tagged `#pin` are grouped at the top. Notes tagged `#archived` are
/// grouped at the bottom with greyed-out styling.
pub(crate) fn note_list<'a>(notes: impl IntoIterator<Item = &'a zk::Note>) -> Markup {
    let mut pinned = Vec::new();
    let mut regular = Vec::new();
    let mut archived = Vec::new();

    for note in notes.into_iter().map(Note::new) {
        match note {
            Note::Pinned(_) => pinned.push(note),
            Note::Regular(_) => regular.push(note),
            Note::Archived(_) => archived.push(note),
        }
    }

    html! {
        @if !pinned.is_empty() {
            @for note in &pinned {
                (note_item(note))
            }
            div class="border-t border-gray-300 dark:border-gray-600" {}
        }

        @for note in &regular {
            (note_item(note))
        }

        @if !archived.is_empty() {
            div class="border-t border-gray-300 dark:border-gray-600" {}
            @for note in &archived {
                (note_item(note))
            }
        }
    }
}

fn note_item(note: &Note) -> Markup {
    let (title_class, snippet_class) = match note {
        Note::Regular(_) | Note::Pinned(_) => (
            "text-md font-semibold text-gray-900 dark:text-white",
            "text-sm text-gray-600 dark:text-gray-300 truncate",
        ),
        Note::Archived(_) => (
            "text-md font-semibold text-gray-400 dark:text-gray-500",
            "text-sm text-gray-400 dark:text-gray-500 truncate",
        ),
    };
    let is_pinned = matches!(note, Note::Pinned(_));
    let note = note.inner();

    html! {
        div
            class="p-4 border-l-4 border-l-transparent border-b border-gray-200 dark:border-gray-700 dark:border-l-transparent cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 hover:border-l-blue-400 active:bg-gray-200 dark:active:bg-gray-600"
            hx-get={ "/f/" (note.filename_stem()) }
            hx-target="#note-content"
            hx-push-url={ "/note/" (note.filename_stem()) }
            onclick="showNote()" {
            div class="flex items-center justify-between" {
                div {
                    h3 class=(title_class) { (note.title()) }
                    p class=(snippet_class) { (note.snippet()) }
                }
                @if is_pinned {
                    span class="ml-2 mr-2 flex-shrink-0" { (icons::pin()) }
                }
            }
        }
    }
}
