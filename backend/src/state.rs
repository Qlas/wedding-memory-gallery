use std::path::PathBuf;

use crate::database::Database;
use crate::errors::AppError;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub storage_directory: PathBuf,
}

impl AppState {
    pub async fn try_new() -> Result<Self, AppError> {
        Ok(Self {
            database: Database::try_new().await?,
            storage_directory: PathBuf::from("storage/"),
        })
    }

    #[cfg(test)]
    pub fn with_storage_directory(mut self, storage: PathBuf) -> Self {
        self.storage_directory = storage;
        self
    }
}
