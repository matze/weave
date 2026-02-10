use axum::http::header;
use axum::response::IntoResponse;

pub(crate) mod icons;

pub(crate) async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/svg+xml")],
        include_str!("favicon.svg"),
    )
}

pub(crate) async fn css() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!(concat!(env!("OUT_DIR"), "/app.css")),
    )
}

pub(crate) async fn htmx_js() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css")],
        include_str!("htmx.2.0.4.min.js"),
    )
}
