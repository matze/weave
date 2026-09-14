use axum::extract::State;
use maud::{Markup, html};

use crate::extract::{Authenticated, LoginDisabled};
use crate::{Notebook, partials};

pub(crate) async fn index(
    State(notebook): State<Notebook>,
    State(LoginDisabled(login_disabled)): State<LoginDisabled>,
    Authenticated(authenticated): Authenticated,
) -> Markup {
    partials::layout::layout(authenticated, login_disabled, notebook, html! {}, false)
}
