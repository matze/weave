use maud::{Markup, html};

use crate::assets::icons;
use crate::zk::{self, NoteExt};

enum Kind {
    Regular,
    Pinned,
    Archived,
}

fn classify(note: &zk::Note) -> Kind {
    if note.has("pin") {
        Kind::Pinned
    } else if note.has("archived") {
        Kind::Archived
    } else {
        Kind::Regular
    }
}

/// Render a list of notes for the sidebar.
///
/// `#pin` notes are grouped at the top, `#archived` at the bottom (greyed out),
/// regular notes in the middle in modified-date order.
pub(crate) fn note_list<'a>(notes: impl IntoIterator<Item = &'a zk::Note>) -> Markup {
    let notes: Vec<&zk::Note> = notes.into_iter().collect();

    html! {
        @for note in notes.iter().filter(|n| matches!(classify(n), Kind::Pinned)) {
            (note_row(note, Kind::Pinned))
        }
        @for note in notes.iter().filter(|n| matches!(classify(n), Kind::Regular)) {
            (note_row(note, Kind::Regular))
        }
        @for note in notes.iter().filter(|n| matches!(classify(n), Kind::Archived)) {
            (note_row(note, Kind::Archived))
        }
    }
}

fn note_row(note: &zk::Note, kind: Kind) -> Markup {
    let row_class = match kind {
        Kind::Regular => "note-row",
        Kind::Pinned => "note-row note-row--pinned",
        Kind::Archived => "note-row note-row--archived",
    };
    let is_pinned = matches!(kind, Kind::Pinned);

    html! {
        div class=(row_class)
            data-stem=(note.filename_stem())
            hx-get={ "/f/" (note.filename_stem()) }
            hx-target="#note-content"
            hx-push-url={ "/note/" (note.filename_stem()) }
            onclick="showNote(event)" {
            div class="nr-top" {
                span class="nr-title" { (note.title()) }
                @if is_pinned {
                    span class="nr-pin" { (icons::pin()) }
                }
            }
            p class="nr-preview" { (note.snippet()) }
        }
    }
}
