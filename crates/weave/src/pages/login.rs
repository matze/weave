use axum::extract::State;
use axum::response::{IntoResponse, Redirect, Response};
use maud::{DOCTYPE, Markup, html};

use crate::extract::LoginDisabled;
use crate::partials;

pub(crate) async fn login(State(LoginDisabled(login_disabled)): State<LoginDisabled>) -> Response {
    if login_disabled {
        return Redirect::to("/").into_response();
    }

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
    .into_response()
}

pub(crate) fn login_failed() -> Markup {
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
                    p class="login-error" role="alert" { "Incorrect password" }
                    button class="btn btn-primary" type="submit" { "Login" }
                }
            }
        }
    }
}
