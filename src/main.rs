mod assets;
mod extract;
mod jwt;
mod md;
mod pages;
mod partials;
mod zk;

use std::sync::{Arc, Mutex, mpsc};

use anyhow::Result;
use axum::Router;
use axum::extract::{Form, FromRef, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, Key};
use futures_concurrency::future::Join;
use notify::event::{AccessKind, AccessMode};
use notify::{EventKind, Watcher};
use serde::Deserialize;

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

async fn watch(notebook: Notebook) -> Result<()> {
    let path = notebook.lock().unwrap().path.clone();

    tokio::task::spawn_blocking(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher = notify::recommended_watcher(move |result| {
            if let Ok(notify::Event { kind, paths, .. }) = result
                && matches!(
                    kind,
                    EventKind::Access(AccessKind::Close(AccessMode::Write))
                )
            {
                for path in paths.into_iter() {
                    if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                        tracing::debug!(?path, "changed");
                        tx.send(path).unwrap();
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
        .route("/", get(pages::index::index))
        .route("/note/{stem}", get(pages::note::note))
        .route("/login", get(pages::login::login).post(do_login))
        .route("/f/search", post(partials::search::search))
        .route("/f/{stem}", get(partials::note::note))
        .route("/app.css", get(assets::css))
        .route("/htmx.2.0.4.min.js", get(assets::htmx_js))
        .with_state(state);

    tracing::info!("serving on localhost:8000");
    let listener = tokio::net::TcpListener::bind("localhost:8000").await?;
    let _ = (watch(notebook), axum::serve(listener, app)).join().await;

    Ok(())
}
