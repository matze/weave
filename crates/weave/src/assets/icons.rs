use maud::{Markup, html};

// Lucide-style stroke icons. 24x24 viewBox, currentColor stroke, 14px display.

fn icon(d_paths: &[&str]) -> Markup {
    html! {
        svg class="icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14"
            viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" {
            @for d in d_paths { path d=(d) {} }
        }
    }
}

pub(crate) fn sign_in() -> Markup {
    icon(&[
        "M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4",
        "M10 17l5-5-5-5",
        "M15 12H3",
    ])
}

pub(crate) fn sign_out() -> Markup {
    icon(&[
        "M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4",
        "M16 17l5-5-5-5",
        "M21 12H9",
    ])
}

pub(crate) fn pencil() -> Markup {
    icon(&["M17 3a2.828 2.828 0 0 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"])
}

pub(crate) fn eye() -> Markup {
    icon(&[
        "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z",
        "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z",
    ])
}

pub(crate) fn search() -> Markup {
    icon(&["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z", "M21 21l-4.35-4.35"])
}

pub(crate) fn link() -> Markup {
    icon(&[
        "M10 13a5 5 0 0 0 7.07 0l3-3a5 5 0 1 0-7.07-7.07l-1.5 1.5",
        "M14 11a5 5 0 0 0-7.07 0l-3 3a5 5 0 1 0 7.07 7.07l1.5-1.5",
    ])
}

pub(crate) fn pin() -> Markup {
    html! {
        svg class="icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14"
            viewBox="0 0 24 24" fill="currentColor" {
            circle cx="12" cy="12" r="3" {}
        }
    }
}

pub(crate) fn plus() -> Markup {
    icon(&["M12 5v14", "M5 12h14"])
}

pub(crate) fn moon() -> Markup {
    icon(&["M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"])
}
