use maud::{Markup, html};

/// Render the `<head>` element with meta tags, styles, and view-state JS.
pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="htmx-config" content=r#"{"refreshOnHistoryMiss":true}"#;
            link rel="stylesheet" type="text/css" href="/app.css";
            link rel="stylesheet" type="text/css" href="/highlight.css";
            link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg";
            title { "weave" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head_html() -> String {
        head().into_string()
    }

    #[test]
    fn test_sync_view_on_page_load() {
        let html = head_html();
        assert!(html.contains("function syncView("), "{html}");
        assert!(html.contains("DOMContentLoaded"), "{html}");
        assert!(html.contains("syncView(true)"), "{html}");
    }

    #[test]
    fn test_sync_view_on_popstate() {
        let html = head_html();
        assert!(html.contains("popstate"), "{html}");
        // popstate should trigger syncView for browser back/forward
        assert!(
            html.contains(r#"addEventListener('popstate', function() { syncView(true)"#),
            "{html}"
        );
    }

    #[test]
    fn test_sync_view_on_htmx_settle() {
        let html = head_html();
        assert!(html.contains("htmx:afterSettle"), "{html}");
        assert!(html.contains("note-content"), "{html}");
        assert!(html.contains("search-list"), "{html}");
    }

    #[test]
    fn test_history_save_cleans_mobile_classes() {
        let html = head_html();
        // Before HTMX caches the page, syncView resets mobile classes so
        // the cached snapshot reflects the URL, not stale toggle state.
        assert!(html.contains("htmx:beforeHistorySave"), "{html}");
    }

    #[test]
    fn test_sync_view_on_history_restore() {
        let html = head_html();
        // After HTMX restores cached HTML on back/forward, re-derive view
        // state from the URL so mobile panel visibility is correct.
        assert!(html.contains("htmx:historyRestore"), "{html}");
    }

    #[test]
    fn test_go_back_with_history_tracking() {
        let html = head_html();
        assert!(html.contains("function goBack()"), "{html}");
        assert!(html.contains("hasAppHistory"), "{html}");
        assert!(html.contains("history.back()"), "{html}");
        // Falls back to index when no app history
        assert!(html.contains("location.href = '/'"), "{html}");
    }

    #[test]
    fn test_htmx_history_miss_config() {
        let html = head_html();
        assert!(html.contains("refreshOnHistoryMiss"), "{html}");
        assert!(html.contains("htmx-config"), "{html}");
    }

    #[test]
    fn test_no_show_sidebar_function() {
        let html = head_html();
        assert!(!html.contains("showSidebar"), "{html}");
    }

    #[test]
    fn test_show_note_defers_panel_switch() {
        let html = head_html();
        // showNote must NOT toggle mobile classes — syncView handles that
        // after content arrives, so the user stays on the sidebar while loading.
        let show_note = html.find("function showNote(").expect("showNote exists");
        let after = &html[show_note..];
        let end = after
            .find("\n                function ")
            .unwrap_or(after.len());
        let body = &after[..end];
        assert!(
            !body.contains("mobile-hidden"),
            "showNote must not toggle panels"
        );
        assert!(
            !body.contains("mobile-visible"),
            "showNote must not toggle panels"
        );
    }
}
