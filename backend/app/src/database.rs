use entity::file;
use entity::file::Entity as File;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, EntityTrait, PaginatorTrait, QueryOrder};
use sea_orm::{ActiveValue::Set, DatabaseConnection};

#[derive(Clone)]
pub struct Database {
    db: sea_orm::DatabaseConnection,
}

impl Database {
    #[cfg(not(test))]
    pub async fn try_new() -> Result<Self, sea_orm::DbErr> {
        let db: DatabaseConnection =
            sea_orm::Database::connect("sqlite://sqlite.db?mode=rwc").await?;

        Database::prepare_database(&db).await?;

        Ok(Self { db })
    }

    #[cfg(test)]
    pub async fn try_new() -> Result<Self, sea_orm::DbErr> {
        let db: DatabaseConnection = sea_orm::Database::connect("sqlite::memory:").await?;

        Database::prepare_database(&db).await?;

        Ok(Self { db })
    }

    pub async fn prepare_database(connection: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Migrator::up(connection, None).await
    }

    pub async fn add_file(&self, path: String, mime: &str) -> Result<(), sea_orm::DbErr> {
        let model = file::ActiveModel {
            path: Set(path),
            mime: Set(mime.to_owned()),
            ..Default::default()
        };

        model.insert(&self.db).await?;

        Ok(())
    }

    pub async fn get_files(
        &self,
        page: u16,
        size: u16,
    ) -> Result<Vec<file::Model>, sea_orm::DbErr> {
        File::find()
            .order_by(file::Column::CreatedAt, sea_orm::Order::Desc)
            .paginate(&self.db, size.into())
            .fetch_page(page.into())
            .await
    }

    pub async fn get_file(&self, id: i32) -> Result<file::Model, sea_orm::DbErr> {
        File::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(format!(
                "File with id={id} not found",
            )))
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
