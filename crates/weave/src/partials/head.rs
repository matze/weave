use maud::{Markup, PreEscaped, html};

/// Render the `<head>` element with meta tags, fonts, styles, and pre-paint theme script.
pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="htmx-config" content=r#"{"refreshOnHistoryMiss":true}"#;
            link rel="preconnect" href="https://fonts.googleapis.com";
            link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
            link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:ital,wght@0,400;0,500;0,600;0,700;1,400;1,500;1,600;1,700&family=JetBrains+Mono:ital,wght@0,400;0,500;0,600;1,400;1,500;1,600&display=swap";
            link rel="stylesheet" type="text/css" href="/app.css";
            link rel="stylesheet" type="text/css" href="/highlight.css";
            link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg";
            title { "weave" }
            script {
                (PreEscaped(
                    "(function(){try{var t=localStorage.getItem('theme');\
                    if(t==='dark'||t==='light')document.documentElement.setAttribute('data-theme',t);}\
                    catch(e){}})();"
                ))
            }
        }
    }
}
