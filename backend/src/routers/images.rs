use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::Response;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use std::num::NonZeroU16;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio_util::io::ReaderStream;

use crate::errors::AppError;

pub fn router() -> Router<PathBuf> {
    Router::new()
        .route("/", get(images))
        .route("/{name}/download", get(download))
}

#[derive(Deserialize)]
struct Pagination {
    size: NonZeroU16,
    page: NonZeroU16,
}

async fn images(
    State(storage_directory): State<PathBuf>,
    Query(params): Query<Pagination>,
) -> Result<Json<Vec<String>>, AppError> {
    let mut entries = fs::read_dir(storage_directory).await?;
    let mut all_files = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        all_files.push(entry.file_name().to_string_lossy().into_owned());
    }

    let start = (params.page.get() - 1) * params.size.get();
    Ok(Json(
        all_files
            .into_iter()
            .skip(start.into())
            .take(params.size.get().into())
            .collect(),
    ))
}

async fn download(
    State(storage_directory): State<PathBuf>,
    Path(name): Path<String>,
) -> Result<Response<Body>, AppError> {
    let file_path = storage_directory.join(name);

    let file = File::open(&file_path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(CONTENT_TYPE, "image/jpeg")
        .header(CONTENT_DISPOSITION, "attachment; filename=\"my_image.jpg\"")
        .body(body)
        .map_err(AppError::from)
}

#[cfg(test)]
mod tests {
    // We want to start the full axum server stack.
    use super::super::app;

    use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use axum_test::TestServer;
    use rstest::rstest;
    use serde_json::json;
    use std::fs::write;
    use tempfile::TempDir;

    #[rstest]
    #[tokio::test]
    async fn read_file_list() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let app = app(path.clone());

        write(path.join("test.txt"), "abc").unwrap();

        let server = TestServer::new(app).unwrap();

        let response = server
            .get("images")
            .add_query_param("page", 1)
            .add_query_param("size", 1)
            .await;

        response.assert_status_ok();

        response.assert_json_contains(&json!(["test.txt"]));
    }

    #[rstest]
    #[tokio::test]
    async fn query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let app = app(path);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images").add_query_param("size", 1).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: missing field `page`");
    }

    #[rstest]
    #[tokio::test]
    async fn invalid_query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let app = app(path);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images").add_query_param("size", 0).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: size: invalid value: integer `0`, expected a nonzero u16");
    }

    #[rstest]
    #[tokio::test]
    async fn download_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let app = app(path.clone());

        write(path.join("test"), "abc").unwrap();

        let server = TestServer::new(app).unwrap();

        let response = server.get("/images/test/download").await;

        response.assert_status_ok();
        response.assert_text_contains("abc");
        response.assert_header(CONTENT_TYPE, "image/jpeg");
        response.assert_header(CONTENT_DISPOSITION, "attachment; filename=\"my_image.jpg\"");
    }
}
