use axum::extract::State;
use maud::{Markup, html};

use crate::extract::Authenticated;
use crate::{Notebook, partials};

pub(crate) async fn index(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
) -> Markup {
    partials::layout::layout(authenticated, notebook, html! {})
}
