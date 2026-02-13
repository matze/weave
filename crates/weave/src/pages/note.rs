use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::{Notebook, partials};

pub(crate) async fn note(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    let content = html! {
        div
            hx-get={ "/f/" (stem) }
            hx-trigger="load"
            hx-swap="outerHTML"
            {}
    };

    partials::layout::layout(authenticated, notebook, content)
}
