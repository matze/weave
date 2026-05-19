use maud::{DOCTYPE, Markup, html};

use crate::partials;

pub(crate) async fn login() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            (partials::head::head())
            body class="login" {
                form class="login-form" action="/login" method="post" {
                    input class="login-input"
                        type="password"
                        name="password"
                        id="password"
                        placeholder="Password";
                    button class="btn btn-primary" type="submit" { "Login" }
                }
            }
        }
    }
}
