use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use dom_smoothie::Readability;
use maud::{Markup, html};
use serde::Deserialize;
use url::Url;

use crate::Notebook;
use crate::assets::icons;
use crate::extract::Authenticated;

#[derive(Deserialize)]
pub(crate) struct ClipRequest {
    url: String,
}

/// A clip that failed, worded for the person who triggered it. The `Display`
/// string is shown verbatim in the drawer, so each variant carries a reason
/// rather than a raw error dump.
#[derive(thiserror::Error, Debug)]
enum ClipError {
    #[error("Enter a full http:// or https:// URL")]
    InvalidUrl,
    #[error("Could not reach the page")]
    Fetch(#[source] reqwest::Error),
    #[error("The page returned HTTP {0}")]
    Status(u16),
    #[error("Could not extract readable content from the page")]
    Extract(#[source] anyhow::Error),
    #[error("Could not save the note")]
    Save(#[source] anyhow::Error),
}

impl ClipError {
    fn status_code(&self) -> StatusCode {
        match self {
            ClipError::InvalidUrl => StatusCode::BAD_REQUEST,
            ClipError::Fetch(_) | ClipError::Status(_) => StatusCode::BAD_GATEWAY,
            ClipError::Extract(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ClipError::Save(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Clip a URL into a new note, blocking until the work is done so the response
/// reflects the real outcome. The body is a status fragment swapped into the
/// drawer; the status code lets the client tell success from failure.
pub(crate) async fn clip(
    Authenticated(authenticated): Authenticated,
    State(notebook): State<Notebook>,
    Form(req): Form<ClipRequest>,
) -> (StatusCode, Markup) {
    if !authenticated {
        return (StatusCode::FORBIDDEN, error_message("Not signed in"));
    }

    match clip_url(&req.url, &notebook).await {
        Ok(title) => (StatusCode::OK, success_message(&title)),
        Err(err) => {
            tracing::error!(?err, url = req.url, "clip failed");
            (err.status_code(), error_message(&err.to_string()))
        }
    }
}

async fn clip_url(url: &str, notebook: &Notebook) -> Result<String, ClipError> {
    let url = url.trim();

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(ClipError::InvalidUrl);
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(ClipError::Fetch)?;

    let response = client
        .get(url)
        .header("User-Agent", "weave-clipper/1.0")
        .send()
        .await
        .map_err(ClipError::Fetch)?;

    let status = response.status();

    if !status.is_success() {
        return Err(ClipError::Status(status.as_u16()));
    }

    let html = response.text().await.map_err(ClipError::Fetch)?;
    let url = url.to_owned();

    // Readability parsing and Markdown conversion are synchronous and
    // CPU-bound, so run them off the async executor.
    let (title, content) = tokio::task::spawn_blocking(move || {
        let parsed_url = Url::parse(&url).map_err(|err| ClipError::Extract(err.into()))?;
        let mut readability = Readability::new(html, Some(url.as_str()), None)
            .map_err(|err| ClipError::Extract(err.into()))?;
        let article = readability
            .parse()
            .map_err(|err| ClipError::Extract(err.into()))?;

        let markdown =
            htmd::convert(&article.content).map_err(|err| ClipError::Save(err.into()))?;
        let title = if article.title.is_empty() {
            parsed_url.host_str().unwrap_or("clipping").to_owned()
        } else {
            article.title
        };

        let content = format!("# {title}\n\n{markdown}\n\n---\n\nReference: {url}");

        Ok::<_, ClipError>((title, content))
    })
    .await
    .map_err(|err| ClipError::Extract(err.into()))??;

    notebook
        .lock()
        .unwrap()
        .create_note(&content)
        .map_err(|err| ClipError::Save(err.into()))?;

    Ok(title)
}

fn success_message(title: &str) -> Markup {
    html! {
        div class="clip-msg is-ok" {
            (icons::check())
            span { "Clipped “" (title) "”" }
        }
    }
}

fn error_message(reason: &str) -> Markup {
    html! {
        div class="clip-msg is-err" {
            (icons::alert())
            span { (reason) }
        }
    }
}
