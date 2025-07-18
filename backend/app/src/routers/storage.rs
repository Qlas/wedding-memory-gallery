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
use tracing::info;

use crate::dto::FileResponse;
use crate::errors::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_storage))
        .route("/{id}/full", get(download_full))
        .route("/{id}/thumbnail", get(download_thumb))
}

#[derive(Deserialize)]
struct Pagination {
    size: NonZeroU16,
    page: NonZeroU16,
}

async fn get_storage(
    State(app_state): State<AppState>,
    Query(params): Query<Pagination>,
) -> Result<Json<Vec<FileResponse>>, AppError> {
    let files = app_state
        .database
        .get_files(params.page.get() - 1, params.size.get())
        .await?;

    Ok(Json(files.into_iter().map(Into::into).collect()))
}

async fn download_full(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response<Body>, AppError> {
    info!("aa");
    let db_file = app_state.database.get_file(id).await?;

    let file = File::open(&db_file.full_path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(CONTENT_TYPE, &db_file.mime)
        .body(body)
        .map_err(AppError::from)
}

async fn download_thumb(
    State(app_state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response<Body>, AppError> {
    let db_file = app_state.database.get_file(id).await?;

    let file = File::open(&db_file.thumb_path).await?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Response::builder()
        .header(CONTENT_TYPE, "image/png")
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
    use image::RgbImage;
    use serde_json::json;
    use std::{
        fs::{File, write},
        io::{BufWriter, Cursor},
        ptr::write_bytes,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn read_file_list() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::builder()
            .with_storage_directory(path.clone())
            .try_build()
            .await
            .unwrap();

        state
            .database
            .add_file("test.txt".to_string(), "test2.txt".to_string(), "abc")
            .await
            .unwrap();

        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server
            .get("storage")
            .add_query_param("page", 1)
            .add_query_param("size", 1)
            .await;

        response.assert_status_ok();

        response.assert_json_contains(&json!([{"id": 1, "mime": "abc"}]));
    }

    #[tokio::test]
    async fn query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::builder()
            .with_storage_directory(path)
            .try_build()
            .await
            .unwrap();
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("storage").add_query_param("size", 1).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: missing field `page`");
    }

    #[tokio::test]
    async fn invalid_query_param() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::builder()
            .with_storage_directory(path)
            .try_build()
            .await
            .unwrap();
        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("storage").add_query_param("size", 0).await;

        response.assert_status_bad_request();
        response.assert_text_contains("Failed to deserialize query string: size: invalid value: integer `0`, expected a nonzero u16");
    }

    #[tokio::test]
    async fn download_file() {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::builder()
            .with_storage_directory(path.clone())
            .try_build()
            .await
            .unwrap();

        state
            .database
            .add_file(
                path.join("test").to_string_lossy().to_string(),
                path.join("test2").to_string_lossy().to_string(),
                "abc",
            )
            .await
            .unwrap();

        let img = RgbImage::new(50, 50);

        let file = File::create(path.join("test")).unwrap();
        let mut writer = BufWriter::new(file);
        img.write_to(&mut writer, image::ImageFormat::Png).unwrap();

        let app = app(state);

        let server = TestServer::new(app).unwrap();

        let response = server.get("/storage/1/full").await;

        response.assert_status_ok();
        response.assert_header(CONTENT_TYPE, "abc");
    }
}
