use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::Response;
use axum::http::header::CONTENT_TYPE;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::num::NonZeroU16;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::errors::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(images))
        .route("/{id}/download", get(download))
}

#[derive(Deserialize)]
struct Pagination {
    size: NonZeroU16,
    page: NonZeroU16,
}

async fn images(
    State(app_state): State<AppState>,
    Query(params): Query<Pagination>,
) -> Result<Json<Vec<i64>>, AppError> {
    let files = app_state
        .database
        .get_files(params.page.get() - 1, params.size.get())
        .await?;

    Ok(Json(files.iter().map(|file| file.id).collect()))
}

async fn download(
    State(app_state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Response<Body>, AppError> {
    let db_file = app_state.database.get_file(id).await?;

    let file = File::open(&db_file.path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(CONTENT_TYPE, &db_file.mime)
        .body(body)
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    // We want to start the full axum server stack.
    use super::super::app;
    use crate::state::AppState;

    use axum::http::header::CONTENT_TYPE;
    use axum_test::TestServer;
    use serde_json::json;
    use std::fs::write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn read_file_list() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::try_new()
            .await
            .unwrap()
            .with_storage_directory(path.clone());

        state
            .database
            .add_file("test.txt".to_string(), "abc")
            .await
            .unwrap();

        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server
            .get("images")
            .add_query_param("page", 1)
            .add_query_param("size", 1)
            .await;

        response.assert_status_ok();

        response.assert_json_contains(&json!([1]));
    }

    #[tokio::test]
    async fn query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::try_new()
            .await
            .unwrap()
            .with_storage_directory(path);
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images").add_query_param("size", 1).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: missing field `page`");
    }

    #[tokio::test]
    async fn invalid_query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::try_new()
            .await
            .unwrap()
            .with_storage_directory(path);
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images").add_query_param("size", 0).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: size: invalid value: integer `0`, expected a nonzero u16");
    }

    #[tokio::test]
    async fn download_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::try_new()
            .await
            .unwrap()
            .with_storage_directory(path.clone());

        state
            .database
            .add_file(path.join("test").to_string_lossy().to_string(), "abc")
            .await
            .unwrap();
        write(path.join("test"), "abc").unwrap();

        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images/1/download").await;

        response.assert_status_ok();
        response.assert_text_contains("abc");
        response.assert_header(CONTENT_TYPE, "abc");
    }
}
