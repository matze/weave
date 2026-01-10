use axum::http::header;
use axum::response::IntoResponse;

pub(crate) mod icons;

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
