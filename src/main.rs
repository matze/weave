use crate::extract::Authenticated;
use anyhow::Result;
use axum::Router;
use axum::extract::{Form, FromRef, Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, Key};
use futures_concurrency::future::Join;
use maud::{DOCTYPE, Markup, html};
use notify::{
    EventKind, Watcher,
    event::{AccessKind, AccessMode},
};
use serde::Deserialize;
use std::sync::{Arc, Mutex, mpsc};

mod extract;
mod jwt;
mod md;
mod zk;

type Notebook = Arc<Mutex<zk::Notebook>>;

pub(crate) type Issuer = Arc<jwt::Issuer>;

#[derive(Clone)]
struct AppState {
    /// The static zk [`Notebook`].
    notebook: Notebook,
    /// JWT issuer
    issuer: Issuer,
    /// Key for signing cookies.
    key: Key,
    /// Login password
    password: String,
}

impl FromRef<AppState> for Notebook {
    fn from_ref(state: &AppState) -> Self {
        state.notebook.clone()
    }
}

impl FromRef<AppState> for Issuer {
    fn from_ref(state: &AppState) -> Self {
        state.issuer.clone()
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
            body class="flex h-screen items-center justify-center bg-white dark:bg-gray-900 text-black dark:text-white" {
                div class="flex flex-col items-center justify-center p-8" {
                    form action="/login" method="post" {
                        input
                            type="password"
                            name="password"
                            id="password"
                            placeholder="Password"
                            class="w-full p-2 rounded bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
    State(state): State<AppState>,
    State(issuer): State<Issuer>,
    Form(login): Form<Login>,
) -> (SignedCookieJar, Redirect) {
    // TODO: use argon2
    if login.password == state.password {
        tracing::info!("successful login");
        let token = issuer.new_token();
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
            body class="flex h-screen bg-white dark:bg-gray-800 text-black dark:text-white" {
                div class="w-80 border-r border-gray-200 dark:border-gray-700 flex flex-col" {
                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
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

                    div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                        input type="search"
                            name="query"
                            placeholder="Search notes..."
                            class="w-full p-2 rounded bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
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
    let Some(note) = notebook.lock().unwrap().note(&stem) else {
        return html! {};
    };

    if !authenticated && !note.has("public") {
        html! {
            div class="h-screen flex items-center justify-center" {
                div class="flex flex-col items-center justify-center p-8" {
                    h2 class="text-xl font-bold" { "access denied" }
                }
            }
        }
    } else {
        html! {
            div class="p-4 border-b border-gray-200 dark:border-gray-700" {
                h2 class="text-xl font-bold dark:text-white" { (note.title) }
            }

            div class="flex-grow p-4 overflow-x-auto" {
                div class="prose max-w-none" {
                    (tokio::task::spawn_blocking(move || md::markdown_to_html(&note.body))
                        .await
                        .expect("join working"))
                }
            }
        }
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
    let notebook = notebook.lock().unwrap();

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
                class="p-4 border-b border-gray-200 dark:border-gray-700 cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700"
                hx-get={ "/f/" (note.filename_stem) }
                hx-target="#note-content" {
                h3 class="text-md font-semibold text-gray-900 dark:text-white" { (note.title) }
                p class="text-sm text-gray-600 dark:text-gray-300 truncate" { (note.snippet()) }
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

async fn watch(notebook: Notebook) -> Result<()> {
    // TODO: do better.
    let path = std::path::PathBuf::from(std::env::var("ZK_NOTEBOOK_DIR")?);

    tokio::task::spawn_blocking(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |result| {
            if let Ok(notify::Event { kind, paths, .. }) = result {
                if matches!(
                    kind,
                    EventKind::Access(AccessKind::Close(AccessMode::Write))
                ) {
                    for path in paths.into_iter() {
                        if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                            tx.send(path).unwrap();
                        }
                    }
                }
            }
        })?;

        watcher.watch(&path, notify::RecursiveMode::NonRecursive)?;

        while let Ok(path) = rx.recv() {
            if let Some(stem) = path.file_stem().and_then(|name| name.to_str()) {
                // TODO: we are locking the entire notebook while loading the new note. Perhaps
                // decouple that.
                notebook.lock().unwrap().reload(stem).unwrap();
            }
        }

        Ok::<_, anyhow::Error>(())
    })
    .await??;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let password = std::env::var("WEAVE_PASSWORD").ok().unwrap_or_default();

    if password.is_empty() {
        tracing::warn!("no password set, login is effectively disabled");
    }

    let notebook = zk::Notebook::load()?;
    let issuer = jwt::Issuer::new()?;
    let key = Key::generate();
    let notebook = Arc::new(Mutex::new(notebook));

    let state = AppState {
        notebook: notebook.clone(),
        issuer: Arc::new(issuer),
        key,
        password,
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
    let _ = (watch(notebook), axum::serve(listener, app)).join().await;

    Ok(())
}
