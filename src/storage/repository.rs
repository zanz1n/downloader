use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, FromRow, IntoArguments,
    Pool, Row, Type,
};
use uuid::Uuid;

use super::{Object, ObjectData};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("object `{0}` not found")]
    NotFound(Uuid),
    #[error("sqlx error while fetching: {0}")]
    GetFailed(sqlx::Error),
    #[error("sqlx error while creating: {0}")]
    CreateFailed(sqlx::Error),
    #[error("sqlx error while updating: {0}")]
    UpdateFailed(sqlx::Error),
    #[error("sqlx error while deleting: {0}")]
    DeleteFailed(sqlx::Error),
}

impl RepositoryError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            RepositoryError::NotFound(..) => StatusCode::NOT_FOUND,
            RepositoryError::GetFailed(..)
            | RepositoryError::CreateFailed(..)
            | RepositoryError::UpdateFailed(..)
            | RepositoryError::DeleteFailed(..) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            RepositoryError::NotFound(..) => 1,
            RepositoryError::GetFailed(..) => 2,
            RepositoryError::CreateFailed(..) => 3,
            RepositoryError::UpdateFailed(..) => 4,
            RepositoryError::DeleteFailed(..) => 6,
        }
    }
}

pub struct ObjectRepository<DB: Database> {
    db: Pool<DB>,
}

impl<DB: Database> Clone for ObjectRepository<DB> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl<DB: Database> ObjectRepository<DB> {
    pub fn new(db: Pool<DB>) -> ObjectRepository<DB> {
        ObjectRepository { db }
    }
}

impl<'r, R: Row> FromRow<'r, R> for Object
where
    &'r str: ColumnIndex<R>,

    Vec<u8>: Decode<'r, R::Database>,
    Vec<u8>: Type<R::Database>,

    i64: Decode<'r, R::Database>,
    i64: Type<R::Database>,

    String: Decode<'r, R::Database>,
    String: Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let id: Vec<u8> = row.try_get("id")?;
        let id: [u8; 16] = id.try_into().map_err(|_| {
            sqlx::Error::Decode("parse `id` uuid out of range".into())
        })?;
        let id = Uuid::from_bytes(id);

        let created_at: i64 = row.try_get("created_at")?;
        let created_at = DateTime::from_timestamp_millis(created_at)
            .ok_or_else(|| {
                sqlx::Error::Decode(
                    "parse `created_at` field gone wrong".into(),
                )
            })?;

        let updated_at: i64 = row.try_get("updated_at")?;
        let updated_at = DateTime::from_timestamp_millis(updated_at)
            .ok_or_else(|| {
                sqlx::Error::Decode(
                    "parse `updated_at` field gone wrong".into(),
                )
            })?;

        let name: String = row.try_get("name")?;
        let mime_type: String = row.try_get("mime_type")?;

        let size: i64 = row.try_get("size")?;
        let size = size.try_into().map_err(|err| {
            sqlx::Error::Decode(format!("parse `size`: {err}").into())
        })?;

        let checksum_256: Vec<u8> = row.try_get("checksum_256")?;
        let checksum_256: [u8; 32] = checksum_256.try_into().map_err(|_| {
            sqlx::Error::Decode(
                "parse `checksum_256` array out of range".into(),
            )
        })?;

        Ok(Self {
            id,
            created_at,
            updated_at,
            data: ObjectData {
                name,
                mime_type,
                size,
                checksum_256,
            },
        })
    }
}

impl<DB> ObjectRepository<DB>
where
    DB: Database,
    for<'a> <DB as sqlx::Database>::Arguments<'a>: IntoArguments<'a, DB>,
    for<'a> &'a Pool<DB>: Executor<'a, Database = DB>,

    for<'r> Object: FromRow<'r, DB::Row>,

    for<'e> Vec<u8>: Encode<'e, DB>,
    for<'e> Vec<u8>: Type<DB>,

    for<'e> i64: Encode<'e, DB>,
    i64: Type<DB>,

    for<'e> String: Encode<'e, DB>,
    String: Type<DB>,
{
    pub async fn get(&self, id: Uuid) -> Result<Object, RepositoryError> {
        sqlx::query_as("SELECT * FROM object WHERE id = $1")
            .bind(id.into_bytes().to_vec())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(
                    %error,
                    "got sqlx error while retrieving object",
                );
                RepositoryError::GetFailed(error)
            })?
            .ok_or(RepositoryError::NotFound(id))
    }

    pub async fn create(
        &self,
        id: Uuid,
        data: ObjectData,
    ) -> Result<Object, RepositoryError> {
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        let size: i64 = data.size.try_into().map_err(|_| {
            RepositoryError::CreateFailed(sqlx::Error::Decode(
                format!("encode `size`: out of range").into(),
            ))
        })?;

        sqlx::query_as(
            "INSERT INTO object \
            (id, created_at, updated_at, name, mime_type, size, checksum_256) \
            VALUES ($1, $2, $3, $4, $5, $6, $7) \
            RETURNING *",
        )
        .bind(id.into_bytes().to_vec())
        .bind(now_ms)
        .bind(now_ms)
        .bind(data.name)
        .bind(data.mime_type)
        .bind(size)
        .bind(data.checksum_256.to_vec())
        .fetch_one(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while creating object");
            RepositoryError::CreateFailed(error)
        })
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: ObjectData,
    ) -> Result<Object, RepositoryError> {
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        sqlx::query_as(
            "UPDATE object \
            SET updated_at = $1, name = $2, mime_type = $3, checksum_256 = $4
            WHERE id = $5 RETURNING *",
        )
        .bind(now_ms)
        .bind(data.name)
        .bind(data.mime_type)
        .bind(data.checksum_256.to_vec())
        .bind(id.into_bytes().to_vec())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while updating object");
            RepositoryError::UpdateFailed(error)
        })?
        .ok_or(RepositoryError::NotFound(id))
    }

    pub async fn delete(&self, id: Uuid) -> Result<Object, RepositoryError> {
        sqlx::query_as("DELETE FROM object WHERE id = $1 RETURNING *")
            .bind(id.into_bytes().to_vec())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(%error, "got sqlx error while deleting object");
                RepositoryError::DeleteFailed(error)
            })?
            .ok_or(RepositoryError::NotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use sqlx::{migrate, Pool, Sqlite};
    use test_log::test;
    use uuid::Uuid;

    use crate::storage::{repository::RepositoryError, ObjectData};

    use super::ObjectRepository;

    fn rand_string() -> String {
        Uuid::new_v4().to_string()
    }

    fn rand_data() -> ObjectData {
        ObjectData {
            name: rand_string(),
            mime_type: mime::TEXT_PLAIN.to_string(),
            size: 0,
            checksum_256: Sha256::new().finalize().into(),
        }
    }

    async fn repository() -> ObjectRepository<Sqlite> {
        let db = Pool::connect("sqlite::memory:").await.unwrap();
        migrate!().run(&db).await.unwrap();

        ObjectRepository::new(db)
    }

    #[test(tokio::test)]
    async fn test_create() {
        let repo = repository().await;

        let data = rand_data();

        let obj = repo.create(Uuid::new_v4(), data.clone()).await.unwrap();
        assert_eq!(data, obj.data, "created data mismatches the provided one");

        let obj = repo.get(obj.id).await.unwrap();
        assert_eq!(data, obj.data, "fetched data mismatches the created one");
    }

    #[test(tokio::test)]
    async fn test_update() {
        let repo = repository().await;

        let data = rand_data();
        let obj = repo.create(Uuid::new_v4(), rand_data()).await.unwrap();
        let id = obj.id;

        let obj = repo.update(obj.id, data.clone()).await.unwrap();
        assert_eq!(data, obj.data, "updated data mismatches the provided one");

        let obj = repo.get(id).await.unwrap();
        assert_eq!(data, obj.data, "fetched data mismatches the updated one");
    }

    #[test(tokio::test)]
    async fn test_delete() {
        let repo = repository().await;

        let data = rand_data();
        let obj = repo.create(Uuid::new_v4(), data.clone()).await.unwrap();
        let id = obj.id;

        let obj = repo.delete(id).await.unwrap();
        assert_eq!(data, obj.data, "updated data mismatches the provided one");

        let res = repo.get(id).await;
        assert!(
            matches!(res, Err(RepositoryError::NotFound(id2)) if id2 == id),
            "expected `ObjectError::NotFound` while fetchings deleted object",
        )
    }
}
