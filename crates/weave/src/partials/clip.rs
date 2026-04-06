use std::io::Cursor;

use axum::Form;
use axum::extract::State;
use axum::http::StatusCode;
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

    let bytes = response.bytes().await?;
    let parsed_url = Url::parse(url)?;
    let mut cursor = Cursor::new(&bytes);
    let product = readability::extractor::extract(&mut cursor, &parsed_url)?;

    let markdown = htmd::convert(&product.content)?;
    let title = if product.title.is_empty() {
        parsed_url.host_str().unwrap_or("clipping").to_owned()
    } else {
        product.title
    };

    let content = format!("# {title}\n\n{markdown}\n\n---\n\nReference: {url}");

    notebook.lock().unwrap().create_note(&content)?;

    Ok(())
}
