use maud::{Markup, PreEscaped, html};

use crate::md;

/// Render the `<head>` element with meta tags, styles, and view-state JS.
pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="htmx-config" content=r#"{"refreshOnHistoryMiss":true}"#;
            link rel="stylesheet" type="text/css" href="/app.css";
            link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg";
            style { (PreEscaped(md::highlight_css())) }
            title { "weave" }
            script {
                (maud::PreEscaped(r#"
                function stemFromUrl() {
                    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
                    return m ? decodeURIComponent(m[2]) : null;
                }
                function syncView(scroll) {
                    var stem = stemFromUrl();
                    document.getElementById('sidebar').classList.toggle('mobile-hidden', !!stem);
                    document.getElementById('note-content').classList.toggle('mobile-visible', !!stem);
                    document.querySelectorAll('.note-item.active-note').forEach(function(el) {
                        el.classList.remove('active-note');
                    });
                    if (stem) {
                        var el = document.querySelector('.note-item[data-stem="' + CSS.escape(stem) + '"]');
                        if (el) {
                            el.classList.add('active-note');
                            if (scroll) el.scrollIntoView({block: 'nearest'});
                        }
                    }
                    var h2 = document.querySelector('#note-content h2');
                    document.title = h2 ? h2.textContent + ' \u{2013} weave' : 'weave';
                }
                function showNote(e) {
                    document.getElementById('sidebar').classList.add('mobile-hidden');
                    document.getElementById('note-content').classList.add('mobile-visible');
                    if (e && e.currentTarget) {
                        document.querySelectorAll('.note-item.active-note').forEach(function(el) {
                            el.classList.remove('active-note');
                        });
                        e.currentTarget.classList.add('active-note');
                    }
                }
                function showList() {
                    document.getElementById('sidebar').classList.remove('mobile-hidden');
                    document.getElementById('note-content').classList.remove('mobile-visible');
                }
                var hasAppHistory = location.pathname === '/';
                document.addEventListener('htmx:pushedIntoHistory', function() { hasAppHistory = true; });
                function goBack() {
                    if (hasAppHistory) {
                        history.back();
                    } else {
                        location.href = '/';
                    }
                }
                document.addEventListener('keydown', function(e) {
                    var t = document.activeElement.tagName;
                    if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
                    if (e.key === 'e') {
                        var btn = document.querySelector('[aria-label="Edit note"]');
                        if (btn) btn.click();
                    } else if (e.key === 'f') {
                        e.preventDefault();
                        document.getElementById('filter-input').focus();
                    }
                });
                document.addEventListener('DOMContentLoaded', function() { syncView(true); });
                window.addEventListener('popstate', function() { syncView(true); });
                document.addEventListener('htmx:beforeHistorySave', function() {
                    syncView(false);
                });
                document.addEventListener('htmx:historyRestore', function() {
                    syncView(true);
                });
                document.addEventListener('htmx:afterSettle', function(e) {
                    if (e.detail.target.id === 'note-content' || e.detail.target.id === 'search-list') {
                        syncView(false);
                    }
                });
                "#))
            }
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
}
