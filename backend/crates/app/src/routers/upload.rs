use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::{Json, Router};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use uuid::Uuid;

use crate::dto::FileResponse;
use crate::errors::AppError;
use crate::state::AppState;
use crate::thumbnail::generate_thumbnail;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(upload))
}

async fn upload(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Vec<FileResponse>>, AppError> {
    let mut resp = Vec::new();
    while let Some(mut field) = multipart.next_field().await? {
        info!("Saving a new file.");

        let storage_file_name = Uuid::new_v4().to_string();
        let path = app_state.full_directory.join(&storage_file_name);
        let thumb_path = app_state.thumbnails_directory.join(&storage_file_name);

        let mut file = File::create(&path).await?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        let file_name = field.file_name().unwrap_or("DEFAULT");
        generate_thumbnail(&path, &thumb_path)?;
        let mime = mime_guess::from_path(file_name).first_or_text_plain();

        resp.push(
            app_state
                .database
                .add_file(
                    path.to_string_lossy().to_string(),
                    thumb_path.to_string_lossy().to_string(),
                    mime.essence_str(),
                )
                .await?
                .into(),
        );
    }

    Ok(Json(resp))
}

#[cfg(test)]
mod tests {
    // We want to start the full axum server stack.
    use super::super::app;
    use crate::state::AppState;

    use axum_test::TestServer;
    use axum_test::multipart::{MultipartForm, Part};
    use image::RgbImage;
    use rstest::{fixture, rstest};
    use std::fs::read_dir;
    use std::io::Cursor;
    use tempfile::TempDir;

    #[fixture]
    fn image_form() -> MultipartForm {
        let img = RgbImage::new(50, 50);

        let mut png_bytes = Vec::new();
        img.write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .unwrap();

        let part = Part::bytes(png_bytes)
            .file_name("test.png")
            .mime_type("image/png");

        MultipartForm::new().add_part("file", part)
    }

    #[rstest]
    #[tokio::test]
    async fn upload_file(image_form: MultipartForm) {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::builder()
            .with_storage_directory(path)
            .try_build()
            .await
            .unwrap();
        let app = app(state.clone());

        let server = TestServer::new(app).unwrap();

        let response = server.post("/upload").multipart(image_form).await;

        response.assert_status_ok();

        let tmp_dir: Vec<_> = read_dir(state.full_directory)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(tmp_dir.len(), 1);

        let file = tmp_dir.first().unwrap().path();

        let db_files = state.database.get_files(0, 5).await.unwrap();
        assert_eq!(db_files.len(), 1);
        let first_file = db_files.first().unwrap();
        assert_eq!(first_file.id, 1);
        assert_eq!(first_file.mime, "image/png");
        assert_eq!(first_file.full_path, file.to_string_lossy().to_string());
    }
}
