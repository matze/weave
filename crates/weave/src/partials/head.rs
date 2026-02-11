use maud::{Markup, PreEscaped, html};

use crate::md;

pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            link rel="stylesheet" type="text/css" href="/app.css";
            link rel="shortcut icon" type="image/svg+xml" href="/favicon.svg";
            style { (PreEscaped(md::highlight_css())) }
            title {"weave"};
            script {
                (maud::PreEscaped(r#"
                function showNote() {
                    document.getElementById('sidebar').classList.add('mobile-hidden');
                    document.getElementById('note-content').classList.add('mobile-visible');
                }
                function showSidebar() {
                    document.getElementById('sidebar').classList.remove('mobile-hidden');
                    document.getElementById('note-content').classList.remove('mobile-visible');
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
                window.addEventListener('popstate', function(e) {
                    var sidebar = document.getElementById('sidebar');
                    if (sidebar && sidebar.classList.contains('mobile-hidden')) {
                        showSidebar();
                        e.stopImmediatePropagation();
                    }
                }, true);
                "#))
            }
        }
    }
}
