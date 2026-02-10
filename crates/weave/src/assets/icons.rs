use maud::{Markup, html};

pub(crate) fn locked() -> Markup {
    html! {
        svg class="w-6 h-6 text-gray-400 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
            path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 14v3m-3-6V7a3 3 0 1 1 6 0v4m-8 0h10a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1Z" {}
        }
    }
}

pub(crate) fn unlocked() -> Markup {
    html! {
        svg class="w-6 h-6 text-gray-400 dark:text-white" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
          path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 14v3m4-6V7a3 3 0 1 1 6 0v4M5 11h10a1 1 0 0 1 1 1v7a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1v-7a1 1 0 0 1 1-1Z" {}
        }
    }
}

pub(crate) fn back() -> Markup {
    html! {
        svg class="w-6 h-6" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
            path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" {}
        }
    }
}

pub(crate) fn pencil() -> Markup {
    html! {
        svg class="w-5 h-5" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24" {
            path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.414 2.586a2 2 0 0 1 2.828 0l1.172 1.172a2 2 0 0 1 0 2.828L8.464 19.536l-5.172 1.293 1.293-5.172L17.414 2.586Z" {}
        }
    }
}
