use maud::{Markup, html};

use crate::zk::Note;

/// Render a list of notes for the sidebar.
pub(crate) fn note_list<'a>(notes: impl IntoIterator<Item = &'a Note>) -> Markup {
    html! {
        @for note in notes {
            div
                class="p-4 border-l-4 border-l-transparent border-b border-gray-200 dark:border-gray-700 cursor-pointer hover:bg-gray-50 dark:hover:bg-gray-700 hover:border-l-blue-400 active:bg-gray-200 dark:active:bg-gray-600"
                hx-get={ "/f/" (note.filename_stem) }
                hx-target="#note-content"
                hx-push-url={ "/note/" (note.filename_stem) }
                onclick="showNote()" {
                h3 class="text-md font-semibold text-gray-900 dark:text-white" { (note.title) }
                p class="text-sm text-gray-600 dark:text-gray-300 truncate" { (note.snippet()) }
            }
        }
    }
}
