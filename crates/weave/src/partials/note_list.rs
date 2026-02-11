use maud::{Markup, html};

use crate::zk::{self, NoteExt};

enum Note<'a> {
    Regular(&'a zk::Note),
    Archived(&'a zk::Note),
}

impl<'a> Note<'a> {
    fn new(note: &'a zk::Note) -> Self {
        if note.has("archived") {
            Self::Archived(note)
        } else {
            Self::Regular(note)
        }
    }

    fn inner(&self) -> &zk::Note {
        match self {
            Self::Regular(n) | Self::Archived(n) => n,
        }
    }
}

/// Render a list of notes for the sidebar.
///
/// Notes tagged `#archived` are grouped at the bottom with greyed-out styling.
pub(crate) fn note_list<'a>(notes: impl IntoIterator<Item = &'a zk::Note>) -> Markup {
    let (regular, archived): (Vec<Note>, Vec<Note>) = notes
        .into_iter()
        .map(Note::new)
        .partition(|n| matches!(n, Note::Regular(_)));

    html! {
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
        Note::Regular(_) => (
            "text-md font-semibold text-gray-900 dark:text-white",
            "text-sm text-gray-600 dark:text-gray-300 truncate",
        ),
        Note::Archived(_) => (
            "text-md font-semibold text-gray-400 dark:text-gray-500",
            "text-sm text-gray-400 dark:text-gray-500 truncate",
        ),
    };
    let note = note.inner();

    html! {
        div
            class="p-4 border-l-4 border-l-transparent border-b border-gray-200 dark:border-gray-700 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 hover:border-l-blue-400 active:bg-gray-200 dark:active:bg-gray-600"
            hx-get={ "/f/" (note.filename_stem()) }
            hx-target="#note-content"
            hx-push-url={ "/note/" (note.filename_stem()) }
            onclick="showNote()" {
            h3 class=(title_class) { (note.title()) }
            p class=(snippet_class) { (note.snippet()) }
        }
    }
}
