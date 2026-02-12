use maud::{DOCTYPE, Markup, html};

use crate::partials;

pub(crate) async fn login() -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head { (partials::head::head(None)) }
            body class="flex h-screen items-center justify-center bg-white dark:bg-gray-900 text-black dark:text-white" {
                div class="flex flex-col items-center justify-center p-8" {
                    form action="/login" method="post" {
                        input
                            type="password"
                            name="password"
                            id="password"
                            placeholder="Password"
                            class="w-full p-2 rounded bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-100 placeholder-gray-500 dark:placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                            {}

                        button
                            type="submit"
                            class="mt-6 px-6 py-2 bg-blue-500 text-white font-semibold rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
                            {"Login"}
                        }
                }
            }
        }
    }
}
