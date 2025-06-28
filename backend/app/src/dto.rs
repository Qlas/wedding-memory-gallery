use entity::file;
use sea_orm::prelude::DateTime;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileResponse {
    id: i32,
    mime: String,
    created_at: DateTime,
}

impl From<file::Model> for FileResponse {
    fn from(model: file::Model) -> Self {
        Self {
            id: model.id,
            mime: model.mime,
            created_at: model.created_at,
        }
    }
}
