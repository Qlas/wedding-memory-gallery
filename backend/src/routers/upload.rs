use std::path::PathBuf;

use axum::Router;
use axum::extract::{Multipart, State};
use axum::routing::post;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tracing::info;
use uuid::Uuid;

use crate::errors::AppError;

pub fn router() -> Router<PathBuf> {
    Router::new().route("/", post(upload))
}

async fn upload(
    State(storage_directory): State<PathBuf>,
    mut multipart: Multipart,
) -> Result<(), AppError> {
    while let Some(mut field) = multipart.next_field().await? {
        if let Some(filename) = field.file_name() {
            info!("Saving '{filename}' file.");
            let mut file = File::create(storage_directory.join(Uuid::new_v4().to_string())).await?;

            while let Some(chunk) = field.chunk().await? {
                file.write_all(&chunk).await?;
            }
            file.flush().await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // We want to start the full axum server stack.
    use super::super::app;

    use axum_test::TestServer;
    use axum_test::multipart::{MultipartForm, Part};
    use rstest::{fixture, rstest};
    use std::fs::{read_dir, read_to_string};
    use tempfile::TempDir;

    #[fixture]
    fn image_form() -> MultipartForm {
        let file_bytes = b"test file content".to_vec();
        let part = Part::bytes(file_bytes)
            .file_name("test.jpg")
            .mime_type("image/jpeg");

        MultipartForm::new()
            .add_text("description", "Test Image")
            .add_part("file", part)
    }

    #[rstest]
    #[tokio::test]
    async fn upload_file(image_form: MultipartForm) {
        let tmp_dir = TempDir::new().unwrap();
        let path = tmp_dir.path().to_path_buf();
        let app = app(path.clone());

        let server = TestServer::new(app).unwrap();

        let response = server.post("/upload").multipart(image_form).await;

        response.assert_status_ok();

        let tmp_dir: Vec<_> = read_dir(path).unwrap().filter_map(Result::ok).collect();
        assert_eq!(tmp_dir.len(), 1);
        let file = tmp_dir.first().unwrap().path();
        let file_content = read_to_string(file).unwrap();
        assert_eq!(file_content, "test file content");
    }
}
