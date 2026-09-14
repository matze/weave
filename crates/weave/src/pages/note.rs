use axum::extract::{Path, State};
use maud::{Markup, html};

use crate::extract::{Authenticated, LoginDisabled};
use crate::{Notebook, partials};

pub(crate) async fn note(
    State(notebook): State<Notebook>,
    State(LoginDisabled(login_disabled)): State<LoginDisabled>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    let content = html! {
        div
            hx-get={ "/f/" (stem) }
            hx-trigger="load"
            hx-target="#note-content"
            hx-swap="innerHTML"
            {}
    };

    partials::layout::layout(authenticated, login_disabled, notebook, content, true)
}
