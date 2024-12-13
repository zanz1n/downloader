use axum::http::StatusCode;
use chrono::Utc;
use sqlx::{Database, Encode, Executor, FromRow, IntoArguments, Pool, Type};
use uuid::Uuid;

use super::{Object, ObjectData};

pub const MAX_LIMIT: u32 = 100;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("object `{0}` not found")]
    NotFound(Uuid),
    #[error("the provided limit {0} is beyond the maximum of {MAX_LIMIT}")]
    LimitOutOfRange(u32),
    #[error("sqlx error: {0}")]
    Sqlx(sqlx::Error),
}

impl RepositoryError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            RepositoryError::NotFound(..) => StatusCode::NOT_FOUND,
            RepositoryError::LimitOutOfRange(..) => StatusCode::BAD_REQUEST,
            RepositoryError::Sqlx(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            RepositoryError::NotFound(..) => 1,
            RepositoryError::LimitOutOfRange(..) => 2,
            RepositoryError::Sqlx(..) => 3,
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

impl<DB> ObjectRepository<DB>
where
    DB: Database,
    for<'a> <DB as sqlx::Database>::Arguments<'a>: IntoArguments<'a, DB>,
    for<'a> &'a Pool<DB>: Executor<'a, Database = DB>,

    for<'r> Object: FromRow<'r, DB::Row>,

    for<'e> &'e [u8]: Encode<'e, DB>,
    for<'e> &'e [u8]: Type<DB>,

    for<'e> i64: Encode<'e, DB>,
    i64: Type<DB>,

    for<'e> String: Encode<'e, DB>,
    String: Type<DB>,
{
    pub async fn get(&self, id: Uuid) -> Result<Object, RepositoryError> {
        sqlx::query_as("SELECT * FROM object WHERE id = $1")
            .bind(id.into_bytes().as_slice())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(
                    %error,
                    "got sqlx error while retrieving object",
                );
                RepositoryError::Sqlx(error)
            })?
            .ok_or(RepositoryError::NotFound(id))
    }

    pub async fn get_all(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Object>, RepositoryError> {
        if limit > MAX_LIMIT {
            return Err(RepositoryError::LimitOutOfRange(limit));
        }

        sqlx::query_as(
            "SELECT * FROM object WHERE rowid > $1 \
            ORDER BY rowid LIMIT $2",
        )
        .bind(offset as i64)
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(
                %error,
                "got sqlx error while retrieving multiple objects",
            );
            RepositoryError::Sqlx(error)
        })
    }

    pub async fn get_by_user(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Object>, RepositoryError> {
        if limit > MAX_LIMIT {
            return Err(RepositoryError::LimitOutOfRange(limit));
        }

        sqlx::query_as(
            "SELECT * FROM object WHERE user_id = $1 \
            ORDER BY rowid LIMIT $2 OFFSET $3",
        )
        .bind(user_id.into_bytes().as_slice())
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(
                %error,
                "got sqlx error while retrieving multiple user objects",
            );
            RepositoryError::Sqlx(error)
        })
    }

    pub async fn create(
        &self,
        id: Uuid,
        user_id: Uuid,
        data: ObjectData,
    ) -> Result<Object, RepositoryError> {
        let now_ms = Utc::now().timestamp_millis();

        let size: i64 = data.size.try_into().map_err(|_| {
            RepositoryError::Sqlx(sqlx::Error::Decode(
                format!("encode `size`: out of range").into(),
            ))
        })?;

        sqlx::query_as(
            "INSERT INTO object \
            (id, user_id, created_at, updated_at, name, mime_type, size, checksum_256) \
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
            RETURNING *",
        )
        .bind(id.into_bytes().as_slice())
        .bind(user_id.into_bytes().as_slice())
        .bind(now_ms)
        .bind(now_ms)
        .bind(data.name)
        .bind(data.mime_type)
        .bind(size)
        .bind(data.checksum_256.as_slice())
        .fetch_one(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while creating object");
            RepositoryError::Sqlx(error)
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
            SET updated_at = $1, name = $2, mime_type = $3, \
            size = $4, checksum_256 = $5 \
            WHERE id = $6 RETURNING *",
        )
        .bind(now_ms)
        .bind(data.name)
        .bind(data.mime_type)
        .bind(data.size as i64)
        .bind(data.checksum_256.as_slice())
        .bind(id.into_bytes().as_slice())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while updating object");
            RepositoryError::Sqlx(error)
        })?
        .ok_or(RepositoryError::NotFound(id))
    }

    pub async fn update_info(
        &self,
        id: Uuid,
        name: String,
        mime_type: String,
    ) -> Result<Object, RepositoryError> {
        let now = Utc::now();
        let now_ms = now.timestamp_millis();

        sqlx::query_as(
            "UPDATE object \
            SET updated_at = $1, name = $2, mime_type = $3
            WHERE id = $4 RETURNING *",
        )
        .bind(now_ms)
        .bind(name)
        .bind(mime_type)
        .bind(id.into_bytes().as_slice())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while updating object");
            RepositoryError::Sqlx(error)
        })?
        .ok_or(RepositoryError::NotFound(id))
    }

    pub async fn delete(&self, id: Uuid) -> Result<Object, RepositoryError> {
        sqlx::query_as("DELETE FROM object WHERE id = $1 RETURNING *")
            .bind(id.into_bytes().as_slice())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(%error, "got sqlx error while deleting object");
                RepositoryError::Sqlx(error)
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

    fn rand_mime() -> String {
        let r = (
            rand::random::<bool>(),
            rand::random::<bool>(),
            rand::random::<bool>(),
        );

        match r {
            (true, true, true) => mime::APPLICATION_JAVASCRIPT,
            (true, true, false) => mime::APPLICATION_JSON,
            (true, false, true) => mime::TEXT_PLAIN,
            (true, false, false) => mime::TEXT_CSS,
            (false, true, true) => mime::IMAGE_PNG,
            (false, true, false) => mime::IMAGE_JPEG,
            (false, false, true) => mime::APPLICATION_PDF,
            (false, false, false) => mime::FONT_WOFF,
        }
        .to_string()
    }

    fn rand_data() -> ObjectData {
        ObjectData {
            name: rand_string(),
            mime_type: rand_mime(),
            size: rand::random::<u32>() as u64,
            checksum_256: Sha256::new()
                .chain_update(rand::random::<[u8; 32]>())
                .finalize()
                .into(),
        }
    }

    async fn repository() -> ObjectRepository<Sqlite> {
        let db = Pool::connect("sqlite::memory:").await.unwrap();
        migrate!().run(&db).await.unwrap();

        ObjectRepository::new(db)
    }

    #[test(tokio::test)]
    async fn test_get_all() {
        const SIZE: usize = 13;

        let repo = repository().await;
        let mut datas = Vec::with_capacity(SIZE);

        for _ in 0..SIZE {
            let id = Uuid::new_v4();
            let data = rand_data();

            datas.push((id, data.clone()));
            repo.create(id, Uuid::new_v4(), data).await.unwrap();
        }

        let all_data = repo.get_all(SIZE as u32, 0).await.unwrap();

        assert!(
            all_data.into_iter().map(|v| (v.id, v.data)).eq(datas),
            "returned data in get_all mismatches the created one"
        );
    }

    #[test(tokio::test)]
    async fn test_get_all_offset() {
        const SIZE: usize = 28;
        const CHUNK_SIZE: usize = 4;

        let repo = repository().await;
        let mut datas = Vec::with_capacity(SIZE);

        for _ in 0..SIZE {
            let id = Uuid::new_v4();
            let data = rand_data();

            datas.push((id, data.clone()));
            repo.create(id, Uuid::new_v4(), data).await.unwrap();
        }

        let mut all_data = Vec::new();

        for i in 0..(SIZE / CHUNK_SIZE) {
            let chunk = repo
                .get_all(CHUNK_SIZE as u32, (CHUNK_SIZE * i) as u32)
                .await
                .unwrap();

            all_data.extend(chunk);
        }

        assert!(
            all_data.into_iter().map(|v| (v.id, v.data)).eq(datas),
            "returned data in get_all mismatches the created one"
        );
    }

    #[test(tokio::test)]
    async fn test_get_by_user() {
        const SIZE: usize = 13;

        let repo = repository().await;
        let mut datas = Vec::with_capacity(SIZE + 3);

        let user_id = Uuid::new_v4();

        for _ in 0..SIZE {
            let id = Uuid::new_v4();
            let data = rand_data();

            datas.push((id, data.clone()));
            repo.create(id, user_id, data).await.unwrap();
        }

        for _ in 0..3 {
            repo.create(Uuid::new_v4(), Uuid::new_v4(), rand_data())
                .await
                .unwrap();
        }

        let all_data = repo.get_by_user(user_id, SIZE as u32, 0).await.unwrap();

        assert!(all_data.into_iter().map(|v| (v.id, v.data)).eq(datas));
    }

    #[test(tokio::test)]
    async fn test_get_by_user_offset() {
        const SIZE: usize = 28;
        const CHUNK_SIZE: usize = 4;

        let repo = repository().await;
        let mut datas = Vec::with_capacity(SIZE);

        let user_id = Uuid::new_v4();

        for _ in 0..SIZE {
            let id = Uuid::new_v4();
            let data = rand_data();

            datas.push((id, data.clone()));
            repo.create(id, user_id, data).await.unwrap();
        }

        let mut all_data = Vec::new();

        for i in 0..(SIZE / CHUNK_SIZE) {
            let chunk = repo
                .get_by_user(
                    user_id,
                    CHUNK_SIZE as u32,
                    (CHUNK_SIZE * i) as u32,
                )
                .await
                .unwrap();

            all_data.extend(chunk);
        }

        assert!(all_data.into_iter().map(|v| (v.id, v.data)).eq(datas));
    }

    #[test(tokio::test)]
    async fn test_create() {
        let repo = repository().await;

        let data = rand_data();

        let id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let old_obj = repo.create(id, user_id, data.clone()).await.unwrap();
        assert_eq!(
            data, old_obj.data,
            "created data mismatches the provided one",
        );

        assert_eq!(old_obj.id, id);
        assert_eq!(old_obj.user_id, user_id);

        let obj = repo.get(old_obj.id).await.unwrap();
        assert_eq!(obj, old_obj, "fetched data mismatches the created one");
    }

    #[test(tokio::test)]
    async fn test_update() {
        let repo = repository().await;

        let data = rand_data();
        let obj = repo
            .create(Uuid::new_v4(), Uuid::new_v4(), rand_data())
            .await
            .unwrap();
        let id = obj.id;

        let mut old_obj = obj.clone();

        let obj = repo.update(obj.id, data.clone()).await.unwrap();
        assert!(
            obj.updated_at > old_obj.updated_at,
            "updated_at field not changed",
        );
        old_obj.updated_at = obj.updated_at;
        old_obj.data = data;

        assert_eq!(obj, old_obj, "updated data mismatches the provided one");

        let obj = repo.get(id).await.unwrap();
        assert_eq!(obj, old_obj, "fetched data mismatches the updated one");
    }

    #[test(tokio::test)]
    async fn test_update_info() {
        let repo = repository().await;

        let data = rand_data();
        let mut old_obj = repo
            .create(Uuid::new_v4(), Uuid::new_v4(), data.clone())
            .await
            .unwrap();

        let new_name = rand_string();
        let new_mime_type = rand_mime();

        let obj = repo
            .update_info(old_obj.id, new_name.clone(), new_mime_type.clone())
            .await
            .unwrap();

        assert!(obj.updated_at > old_obj.updated_at);

        old_obj.data.name = new_name;
        old_obj.data.mime_type = new_mime_type;
        old_obj.updated_at = obj.updated_at;

        assert_eq!(obj, old_obj);

        let obj = repo.get(old_obj.id).await.unwrap();
        assert_eq!(obj, old_obj);
    }

    #[test(tokio::test)]
    async fn test_delete() {
        let repo = repository().await;

        let id = Uuid::new_v4();
        let res = repo.delete(id).await;
        assert!(
            matches!(res, Err(RepositoryError::NotFound(id2)) if id2 == id),
            "expected not found error while deleting non existent object",
        );

        let data = rand_data();
        repo.create(id, Uuid::new_v4(), data.clone()).await.unwrap();

        let obj = repo.delete(id).await.unwrap();
        assert_eq!(data, obj.data, "fetched data mismatches the created one");

        let res = repo.get(id).await;
        assert!(
            matches!(res, Err(RepositoryError::NotFound(id2)) if id2 == id),
            "expected `ObjectError::NotFound` while fetching deleted object",
        )
    }
}
