use axum::Router;
use axum::body::Body;
use axum::extract::DefaultBodyLimit;
use axum::http::{Method, Request, StatusCode, Uri};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{Span, info, warn};

use crate::state::AppState;

mod storage;
mod upload;

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        // WARNING: Do not leave it like that.
        .allow_origin(Any);

    Router::new()
        .nest("/upload", upload::router())
        .nest("/storage", storage::router())
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024)) // 5 MB
        .layer(TraceLayer::new_for_http().on_request(request_layer))
        .layer(cors)
        .fallback(fallback)
        .with_state(state)
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
