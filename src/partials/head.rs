use maud::{Markup, html};

pub(crate) fn head() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            link rel="stylesheet" type="text/css" href="/app.css";
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
                "#))
            }
        }
    }
}
