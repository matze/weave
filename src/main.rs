use anyhow::Result;
use axum::Router;
use axum::extract::{Form, FromRef, FromRequestParts, Path, State};
use axum::http::header;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, Key};
use jsonwebtoken as jwt;
use maud::{DOCTYPE, Markup, html};
use ring::signature::{Ed25519KeyPair, KeyPair};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;

mod md;
mod zk;

const JWT_SUB: &str = "user";
const JWT_ISS: &str = "weave";

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    iss: String,
    exp: u64,
}

struct JwtInner {
    password: String,
    encoding_key: jwt::EncodingKey,
    decoding_key: jwt::DecodingKey,
    header: jwt::Header,
    validation: jwt::Validation,
    claims: Claims,
}

#[derive(Debug)]
struct Authenticated(bool);

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
    Key: FromRef<S>,
    Jwt: FromRef<S>,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = SignedCookieJar::<Key>::from_request_parts(parts, state).await;
        let jwt = Jwt::from_ref(state);

        let authenticated = jar
            .map(|jar| {
                jar.get("jwt").and_then(|cookie| {
                    jwt::decode::<Claims>(
                        cookie.value_trimmed(),
                        &jwt.decoding_key,
                        &jwt.validation,
                    )
                    .ok()
                })
            })
            .ok()
            .flatten()
            .and_then(|data| {
                (data.claims.sub == JWT_SUB && data.claims.iss == JWT_ISS)
                    .then_some(Authenticated(true))
            })
            .unwrap_or(Authenticated(false));

        Ok(authenticated)
    }
}

type Notebook = Arc<zk::Notebook>;

type Jwt = Arc<JwtInner>;

#[derive(Clone)]
struct AppState {
    /// The static zk [`Notebook`].
    notebook: Notebook,
    /// All things required to issue or validate a token.
    jwt: Jwt,
    /// Key for signing cookies.
    key: Key,
}

impl FromRef<AppState> for Notebook {
    fn from_ref(state: &AppState) -> Self {
        state.notebook.clone()
    }
}

impl FromRef<AppState> for Jwt {
    fn from_ref(state: &AppState) -> Self {
        state.jwt.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            link rel="stylesheet" type="text/css" href="app.css";
            title {"weave notes"};
        }
    }
}

async fn login() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head { (head()) }
            body class="flex h-screen items-center justify-center" {
                div class="flex flex-col items-center justify-center bg-white p-8" {
                    form action="/login" method="post" {
                        input
                            type="password"
                            name="password"
                            id="password"
                            placeholder="Password"
                            class="px-4 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            {}

                        button
                            type="submit"
                            class="mt-6 px-6 py-2 bg-blue-500 text-white font-semibold rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                            {"Login"}
                        }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct Login {
    password: String,
}

async fn do_login(
    jar: SignedCookieJar,
    State(jwt): State<Jwt>,
    Form(login): Form<Login>,
) -> (SignedCookieJar, Redirect) {
    // TODO: use argon2
    if login.password == jwt.password {
        tracing::info!("successful login");
        let token = jsonwebtoken::encode(&jwt.header, &jwt.claims, &jwt.encoding_key).unwrap();
        let cookie = Cookie::build(("jwt", token)).build();
        (jar.add(cookie), Redirect::to("/"))
    } else {
        tracing::warn!("failed login attempt");
        (jar.remove("jwt"), Redirect::to("/"))
    }
}

fn locked_icon() -> Markup {
    html! {
        svg class="w-6 h-6 text-gray-400 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
            path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 14v3m-3-6V7a3 3 0 1 1 6 0v4m-8 0h10a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1Z" {}
        }
    }
}

fn unlocked_icon() -> Markup {
    html! {
        svg class="w-6 h-6 text-gray-400 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
          path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14v3m4-6V7a3 3 0 1 1 6 0v4M5 11h10a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1Z" {}
        }
    }
}

async fn index(Authenticated(authenticated): Authenticated) -> Markup {
    let icon = if authenticated {
        unlocked_icon()
    } else {
        locked_icon()
    };

    html! {
        (DOCTYPE)
        html lang="en" {
            head { (head()) }
            body class="flex h-screen" {
                div class="w-80 bg-white border-r border-gray-200 flex flex-col" {
                    div class="p-4 border-b border-gray-200" {
                        div class="flex" {
                            div class="flex flex-col flex-auto" {
                                span class="text-lg font-bold text-transparent bg-clip-text bg-gradient-to-r from-sky-600 to-green-600" {"weave notes"}
                            }

                            div class="flex flex-col flex-none" {
                                a href="/login" {
                                    (icon)
                                }
                            }
                        }
                    }

                    div class="p-4 border-b border-gray-200" {
                        input type="search"
                            name="query"
                            placeholder="Search notes..."
                            class="w-full p-2 rounded bg-gray-100 text-gray-800 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            hx-post="/f/search"
                            hx-trigger="input changed delay:300ms, keyup[key=='Enter']"
                            hx-target="#search-list"
                            {}
                    }

                    div class="flex-grow overflow-y-auto" id="search-list" {}
                }

                div class="flex-grow flex flex-col overflow-y-auto" id="note-content" {}

                script src="/htmx.2.0.4.min.js" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" {}
            }
        }
    }
}

/// Return note content fragment consisting of title div and rendered markdown.
async fn note(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Path(stem): Path<String>,
) -> Markup {
    if let Some(note) = notebook.note(&stem) {
        if !authenticated && !note.has("public") {
            html! {
                div class="h-screen flex items-center justify-center" {
                    div class="flex flex-col items-center justify-center bg-white p-8" {
                        h2 class="text-xl font-bold" { "access denied" }
                    }
                }
            }
        } else {
            let markdown = note.body.clone();

            html! {
                div class="p-4 border-b border-gray-200 bg-white" {
                    h2 class="text-xl font-bold" { (note.title) }
                }

                div class="flex-grow p-4 overflow-x-auto" {
                    div class="prose max-w-none" {
                        (tokio::task::spawn_blocking(move || md::markdown_to_html(&markdown))
                            .await
                            .expect("join working"))
                    }
                }
            }
        }
    } else {
        html! {}
    }
}

#[derive(Deserialize, Debug)]
struct Search {
    query: String,
}

/// Return fragment for the sidebar search results.
#[tracing::instrument(skip(notebook))]
async fn search(
    State(notebook): State<Notebook>,
    Authenticated(authenticated): Authenticated,
    Form(search): Form<Search>,
) -> Markup {
    // Oh no, blocking ...
    let query = search.query.trim();

    let notes = if let Some(tag) = query.strip_prefix('#') {
        if !authenticated {
            notebook.search_tag("public")
        } else {
            // Split multiple tags and compute union
            notebook.search_tag(tag)
        }
    } else {
        notebook.search_titles(&search.query, (!authenticated).then_some("public"))
    };

    tracing::info!(number = notes.len(), "search results");

    html! {
        @for note in notes {
            div
                class="p-4 border-b border-gray-200 cursor-pointer hover:bg-gray-100"
                hx-get={ "/f/" (note.filename_stem) }
                hx-target="#note-content" {
                h3 class="text-md font-semibold" { (note.title) }
                p class="text-sm text-gray-600 truncate" { (note.snippet()) }
            }
        }
    }
}

async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!(concat!(env!("OUT_DIR"), "/app.css")),
    )
}

async fn htmx_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("assets/htmx.2.0.4.min.js"),
    )
}

impl JwtInner {
    fn new(password: String) -> Result<Self> {
        let key_pair = Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new())?;
        let encoding_key = jwt::EncodingKey::from_ed_der(key_pair.as_ref());

        let key_pair = Ed25519KeyPair::from_pkcs8(key_pair.as_ref())?;
        let decoding_key = jwt::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

        let claims = Claims {
            sub: "user".into(),
            iss: "weave".into(),
            exp: jwt::get_current_timestamp() + 60 * 24 * 30,
        };

        let header = jwt::Header::new(jwt::Algorithm::EdDSA);
        let validation = jwt::Validation::new(jwt::Algorithm::EdDSA);

        Ok(Self {
            password,
            encoding_key,
            decoding_key,
            header,
            validation,
            claims,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let password = std::env::var("WEAVE_PASSWORD").ok().unwrap_or_default();

    if password.is_empty() {
        tracing::warn!("no password set, login is effectively disabled");
    }

    let notebook = zk::Notebook::load()?;
    let jwt = JwtInner::new(password)?;
    let key = Key::generate();

    let state = AppState {
        notebook: Arc::new(notebook),
        jwt: Arc::new(jwt),
        key,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/login", get(login).post(do_login))
        .route("/f/search", post(search))
        .route("/f/{stem}", get(note))
        .route("/app.css", get(css))
        .route("/htmx.2.0.4.min.js", get(htmx_js))
        .with_state(state);

    tracing::info!("serving on localhost:8000");
    let listener = tokio::net::TcpListener::bind("localhost:8000").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
