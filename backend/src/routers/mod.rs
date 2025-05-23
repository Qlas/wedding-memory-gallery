use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, Uri};
use tower_http::trace::TraceLayer;
use tracing::{Span, info, warn};

mod images;
mod upload;

pub fn app(storage_directory: PathBuf) -> Router {
    Router::new()
        .nest("/upload", upload::router())
        .nest("/images", images::router())
        .layer(TraceLayer::new_for_http().on_request(request_layer))
        .fallback(fallback)
        .with_state(storage_directory)
}

async fn fallback(method: Method, uri: Uri) -> StatusCode {
    warn!(
        method = ?method,
        path = uri.path(),
        "Not Found"
    );

    StatusCode::NOT_FOUND
}

fn request_layer(request: &Request<Body>, _span: &Span) {
    info!(
        method = %request.method(),
        path = %request.uri().path(),
        "Incoming HTTP request"
    );
}
