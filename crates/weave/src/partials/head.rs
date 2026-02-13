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
            title { "weave" }
            script {
                (maud::PreEscaped(r#"
                function stemFromUrl() {
                    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
                    return m ? decodeURIComponent(m[2]) : null;
                }
                function highlightActiveNote(scroll) {
                    var stem = stemFromUrl();
                    document.querySelectorAll('.note-item.active-note').forEach(function(el) {
                        el.classList.remove('active-note');
                    });
                    if (!stem) return;
                    var el = document.querySelector('.note-item[data-stem="' + CSS.escape(stem) + '"]');
                    if (el) {
                        el.classList.add('active-note');
                        if (scroll) el.scrollIntoView({block: 'nearest'});
                    }
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
                document.addEventListener('DOMContentLoaded', function() {
                    highlightActiveNote(true);
                });
                window.addEventListener('popstate', function(e) {
                    var sidebar = document.getElementById('sidebar');
                    if (sidebar && sidebar.classList.contains('mobile-hidden')) {
                        showSidebar();
                        e.stopImmediatePropagation();
                    }
                    highlightActiveNote(true);
                }, true);
                document.addEventListener('htmx:afterSettle', function(e) {
                    if (e.detail.target.id === 'note-content') {
                        var h2 = document.querySelector('#note-content h2');
                        document.title = h2 ? h2.textContent + ' \u2013 weave' : 'weave';
                        highlightActiveNote(false);
                    }
                    if (e.detail.target.id === 'search-list') {
                        highlightActiveNote(false);
                    }
                });
                "#))
            }
        }
    }
}
