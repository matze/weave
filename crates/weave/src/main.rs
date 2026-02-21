mod assets;
mod extract;
mod jwt;
mod md;
mod pages;
mod partials;
mod zk;

use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, mpsc};

use anyhow::Result;
use axum::Router;
use axum::extract::{Form, FromRef, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, Key};
use futures_concurrency::future::Join;
use notify::event::{AccessKind, AccessMode, ModifyKind, RenameMode};
use notify::{EventKind, Watcher};
use serde::Deserialize;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::trace::TraceLayer;

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

enum WatchEvent {
    Modified(std::path::PathBuf),
    Removed(std::path::PathBuf),
}

async fn watch(notebook: Notebook) -> Result<()> {
    let path = notebook.lock().unwrap().path.clone();

    tokio::task::spawn_blocking(move || {
        let (tx, rx) = mpsc::channel::<WatchEvent>();

        let mut watcher = notify::recommended_watcher(move |result| {
            let Ok(notify::Event { kind, paths, .. }) = result else {
                return;
            };

            let is_md = |p: &std::path::Path| p.extension().is_some_and(|e| e == "md");

            match kind {
                // File deleted outright, or a note was renamed away.
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    for path in paths {
                        if is_md(&path) {
                            tracing::debug!(?path, "removed");
                            let _ = tx.send(WatchEvent::Removed(path));
                        }
                    }
                }
                // Atomic rename (both paths in one event): first is old, second is new.
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                    if let [from, to] = paths.as_slice() {
                        if is_md(from) {
                            tracing::debug!(?from, "renamed away");
                            let _ = tx.send(WatchEvent::Removed(from.clone()));
                        }
                        if is_md(to) {
                            tracing::debug!(?to, "renamed to");
                            let _ = tx.send(WatchEvent::Modified(to.clone()));
                        }
                    }
                }
                // Normal write/create/rename-to.
                EventKind::Access(AccessKind::Close(AccessMode::Write))
                | EventKind::Create(_)
                | EventKind::Modify(ModifyKind::Name(_)) => {
                    for path in paths {
                        if is_md(&path) {
                            tracing::debug!(?path, "changed");
                            let _ = tx.send(WatchEvent::Modified(path));
                        }
                    }
                }
                _ => {}
            }
        })?;

        // Watch recursively so notes in subdirectories are also tracked.
        watcher.watch(&path, notify::RecursiveMode::Recursive)?;

        while let Ok(event) = rx.recv() {
            match event {
                WatchEvent::Modified(path) => {
                    if let Some(stem) = path.file_stem().and_then(|n| n.to_str()) {
                        // TODO: we are locking the entire notebook while loading the new note.
                        // Perhaps decouple that.
                        if let Err(err) = notebook.lock().unwrap().reload(stem) {
                            tracing::error!(?err, "failed to reload {stem}");
                        }
                    }
                }
                WatchEvent::Removed(path) => {
                    if let Some(stem) = path.file_stem().and_then(|n| n.to_str()) {
                        tracing::info!(stem, "removing note");
                        notebook.lock().unwrap().remove(stem);
                    }
                }
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

    let port = std::env::var("WEAVE_PORT")
        .ok()
        .map(|port| port.parse::<u16>())
        .transpose()?
        .unwrap_or(8000);

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
        .route("/", get(pages::index::index))
        .route("/note/{stem}", get(pages::note::note))
        .route("/login", get(pages::login::login).post(do_login))
        .route("/f/search", post(partials::search::search))
        .route(
            "/f/{stem}",
            get(partials::note::note).put(partials::edit::save),
        )
        .route("/f/{stem}/edit", get(partials::edit::edit))
        .route("/f/{stem}/preview", post(partials::edit::preview))
        .route("/app.css", get(assets::app_css))
        .route("/app.js", get(assets::app_js))
        .route("/favicon.svg", get(assets::favicon))
        .route("/highlight.css", get(assets::highlight_css))
        .route("/htmx.2.0.4.min.js", get(assets::htmx_js))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(state);

    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("serving on {addr:?}");
    let _ = (watch(notebook), axum::serve(listener, app)).join().await;

    Ok(())
}
