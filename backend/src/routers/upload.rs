use axum::Router;
use axum::extract::{Multipart, State};
use axum::routing::post;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use uuid::Uuid;

use crate::errors::AppError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(upload))
}

async fn upload(
    State(app_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(), AppError> {
    while let Some(mut field) = multipart.next_field().await? {
        info!("Saving a new file.");

        let path = app_state.storage_directory.join(Uuid::new_v4().to_string());

        let mut file = File::create(path.clone()).await?;

        while let Some(chunk) = field.chunk().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        let file_name = field.file_name().unwrap_or("DEFAULT");
        let mime = mime_guess::from_path(file_name).first_or_text_plain();

        app_state
            .database
            .add_file(path.to_string_lossy().to_string(), mime.essence_str())
            .await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // We want to start the full axum server stack.
    use super::super::app;
    use crate::state::AppState;

    use axum_test::TestServer;
    use axum_test::multipart::{MultipartForm, Part};
    use rstest::{fixture, rstest};
    use std::fs::{read_dir, read_to_string};
    use tempfile::TempDir;

    #[fixture]
    fn image_form() -> MultipartForm {
        let file_bytes = b"test file content".to_vec();
        let part = Part::bytes(file_bytes)
            .file_name("test.png")
            .mime_type("image/jpeg");

        MultipartForm::new().add_part("file", part)
    }

    #[rstest]
    #[tokio::test]
    async fn upload_file(image_form: MultipartForm) {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let state = AppState::try_new()
            .await
            .unwrap()
            .with_storage_directory(path);
        let app = app(state.clone());

        let server = TestServer::new(app).unwrap();

        let response = server.post("/upload").multipart(image_form).await;

        response.assert_status_ok();

        let tmp_dir: Vec<_> = read_dir(state.storage_directory)
            .unwrap()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(tmp_dir.len(), 1);

        let file = tmp_dir.first().unwrap().path();
        let file_content = read_to_string(file.clone()).unwrap();
        assert_eq!(file_content, "test file content");

        let db_files = state.database.get_files(0, 5).await.unwrap();
        assert_eq!(db_files.len(), 1);
        let first_file = db_files.first().unwrap();
        assert_eq!(first_file.id, 1);
        assert_eq!(first_file.mime, "image/png");
        assert_eq!(first_file.path, file.to_string_lossy().to_string());
    }
}
