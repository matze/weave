use maud::{Markup, html};

pub(crate) fn cancel() -> Markup {
    html! {
        svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="-1 -1 26 26" {
            path stroke="currentColor" d="m16.535,8.172l-3.828,3.828,3.828,3.828-.707.707-3.828-3.828-3.828,3.828-.707-.707,3.828-3.828-3.828-3.828.707-.707,3.828,3.828,3.828-3.828.707.707Zm7.465,3.828c0,6.617-5.383,12-12,12S0,18.617,0,12,5.383,0,12,0s12,5.383,12,12Zm-1,0c0-6.065-4.935-11-11-11S1,5.935,1,12s4.935,11,11,11,11-4.935,11-11Z" {}
        }
    }
}

pub(crate) fn sign_in() -> Markup {
    html! {
        svg class="w-5 h-5 text-gray-400 dark:text-white hover:text-gray-600 dark:hover:text-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {
          path stroke="currentColor" d="m24,2.5v19c0,1.448-1.051,2.5-2.5,2.5H2.5c-1.449,0-2.5-1.052-2.5-2.5v-6.5h1v6.5c0,.897.603,1.5,1.5,1.5h19c.897,0,1.5-.603,1.5-1.5V2.5c0-.897-.603-1.5-1.5-1.5H2.5c-.897,0-1.5.603-1.5,1.5v6.5H0V2.5C0,1.051,1.051,0,2.5,0h19c1.449,0,2.5,1.051,2.5,2.5Zm-6.288,10.202l-4.561,4.439.697.717,4.565-4.444c.378-.377.586-.88.586-1.414s-.208-1.037-.589-1.417l-4.561-4.478-.7.714,4.557,4.475c.06.06.096.136.139.207H0v1h17.847c-.041.07-.076.144-.135.202Z" {}
        }
    }
}

pub(crate) fn sign_out() -> Markup {
    html! {
        svg class="w-5 h-5 text-gray-400 dark:text-white hover:text-gray-600 dark:hover:text-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {
          path stroke="currentColor" d="m24,12c0,.534-.208,1.037-.586,1.414l-4.565,4.444-.697-.717,4.561-4.439c.058-.058.093-.132.135-.202H6v-1h16.846c-.042-.071-.078-.147-.139-.207l-4.556-4.435.697-.717,4.561,4.439c.383.382.591.885.591,1.419Zm-13,9.5c0,.827-.673,1.5-1.5,1.5H2.5c-.827,0-1.5-.673-1.5-1.5V2.5c0-.827.673-1.5,1.5-1.5h7c.827,0,1.5.673,1.5,1.5v6.5h1V2.5c0-1.378-1.122-2.5-2.5-2.5H2.5C1.122,0,0,1.122,0,2.5v19c0,1.379,1.122,2.5,2.5,2.5h7c1.378,0,2.5-1.121,2.5-2.5v-6.5h-1v6.5Z" {}
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
        svg class="w-5 h-5 text-gray-400 dark:text-white hover:text-gray-600 dark:hover:text-gray-200" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" {
            path stroke="currentColor" d="M22.94,1.061c-1.368-1.367-3.76-1.365-5.124,0L0,18.876v5.124H5.124L22.94,6.184c.684-.684,1.06-1.593,1.06-2.562s-.376-1.878-1.06-2.562ZM4.71,23H1v-3.71L15.292,4.999l3.709,3.709L4.71,23ZM22.233,5.477l-2.525,2.525-3.709-3.709,2.525-2.525c.986-.988,2.718-.99,3.709,0,.495,.495,.767,1.153,.767,1.854s-.272,1.359-.767,1.854Z" {}
        }
    }
}
