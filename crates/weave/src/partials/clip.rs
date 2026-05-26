use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
use dom_smoothie::Readability;
use serde::Deserialize;
use url::Url;

use crate::Notebook;
use crate::extract::Authenticated;

#[derive(Deserialize)]
pub(crate) struct ClipRequest {
    url: String,
}

pub(crate) async fn clip(
    Authenticated(authenticated): Authenticated,
    State(notebook): State<Notebook>,
    Form(req): Form<ClipRequest>,
) -> StatusCode {
    if !authenticated {
        return StatusCode::FORBIDDEN;
    }

    let url = req.url.trim().to_owned();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return StatusCode::BAD_REQUEST;
    }

    tokio::spawn(async move {
        if let Err(err) = clip_url(&url, &notebook).await {
            tracing::error!(?err, url, "clip failed");
        }
    });

    StatusCode::ACCEPTED
}

async fn clip_url(url: &str, notebook: &Notebook) -> anyhow::Result<()> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(url)
        .header("User-Agent", "weave-clipper/1.0")
        .send()
        .await?;

    let html = response.text().await?;
    let parsed_url = Url::parse(url)?;
    let mut readability = Readability::new(html, Some(url), None)?;
    let article = readability.parse()?;

    let markdown = htmd::convert(&article.content)?;
    let title = if article.title.is_empty() {
        parsed_url.host_str().unwrap_or("clipping").to_owned()
    } else {
        article.title
    };

    let content = format!("# {title}\n\n{markdown}\n\n---\n\nReference: {url}");

    notebook.lock().unwrap().create_note(&content)?;

    Ok(())
}
