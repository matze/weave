use maud::{Markup, html};

/// Render the `<head>` element with meta tags, styles, and view-state JS.
pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="htmx-config" content=r#"{"refreshOnHistoryMiss":true}"#;
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:ital,wght@0,100..900;1,100..900&display=swap";
            link rel="stylesheet" type="text/css" href="/app.css";
            link rel="stylesheet" type="text/css" href="/highlight.css";
            link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg";
            title { "weave" }
        }
    }
}
