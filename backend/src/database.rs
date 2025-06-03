use chrono::NaiveDateTime;
use sqlx::SqlitePool;

#[cfg(not(test))]
use sqlx::sqlite::SqliteConnectOptions;
#[cfg(not(test))]
use std::str::FromStr;

pub struct File {
    pub id: i64,
    pub mime: String,
    pub path: String,
    #[allow(dead_code)]
    pub created_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    #[cfg(not(test))]
    pub async fn try_new() -> Result<Self, sqlx::Error> {
        let db_options = SqliteConnectOptions::from_str("sqlite.db")?.create_if_missing(true);

        let pool = SqlitePool::connect_with(db_options).await?;

        Database::prepare_database(&pool).await?;

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub async fn try_new() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(":memory:").await?;

        Database::prepare_database(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn prepare_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations").run(pool).await?;

        Ok(())
    }

    pub async fn add_file(&self, path: String, mime: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO files (path, mime) VALUES (?, ?)")
            .bind(&path)
            .bind(mime)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_files(&self, page: u16, size: u16) -> Result<Vec<File>, sqlx::Error> {
        sqlx::query_as!(
            File,
            "SELECT * FROM files ORDER BY created_at DESC LIMIT ? OFFSET ?",
            size,
            page
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn get_file(&self, id: i64) -> Result<File, sqlx::Error> {
        sqlx::query_as!(File, "SELECT * FROM files WHERE id=?", id)
            .fetch_one(&self.pool)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn adding_file() {
        let db = Database::try_new().await.unwrap();

        db.add_file("test".to_string(), "xyz").await.unwrap();

        let files = db.get_files(0, 5).await.unwrap();
        assert_eq!(files.len(), 1);
        let first_file = files.first().unwrap();
        assert_eq!(first_file.id, 1);
        assert_eq!(first_file.mime, "xyz");
        assert_eq!(first_file.path, "test".to_string());
    }

    #[tokio::test]
    async fn get_one_file() {
        let db = Database::try_new().await.unwrap();

        db.add_file("test".to_string(), "xyz").await.unwrap();

        let file = db.get_file(1).await.unwrap();
        assert_eq!(file.id, 1);
        assert_eq!(file.mime, "xyz");
        assert_eq!(file.path, "test".to_string());
    }
}
