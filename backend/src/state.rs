use std::path::PathBuf;

use tokio::fs;

use crate::database::Database;
use crate::errors::AppError;

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub storage_directory: PathBuf,
}

impl AppState {
    async fn try_new(storage_directory: PathBuf) -> Result<Self, AppError> {
        let state = Self {
            database: Database::try_new().await?,
            storage_directory,
        };

        fs::create_dir_all(&state.storage_directory).await?;

        Ok(state)
    }

    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }
}

#[derive(Default)]
pub struct AppStateBuilder {
    storage_directory: Option<PathBuf>,
}

impl AppStateBuilder {
    #[cfg(test)]
    pub fn with_storage_directory(mut self, dir: PathBuf) -> Self {
        self.storage_directory = Some(dir);

        self
    }

    pub async fn try_build(self) -> Result<AppState, AppError> {
        AppState::try_new(self.storage_directory.unwrap_or(PathBuf::from("storage/"))).await
    }
}
