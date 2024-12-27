use chrono::Utc;
use sqlx::{
    ColumnIndex, Database, Decode, Encode, Executor, FromRow, IntoArguments,
    Pool, Row, Type,
};
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::auth::Permission;

use super::{User, UserData, UserError};

struct UserWithPassword {
    pub user: User,
    pub password_hash: String,
}

impl<'r, R: Row> FromRow<'r, R> for UserWithPassword
where
    User: FromRow<'r, R>,

    &'r str: ColumnIndex<R>,
    String: Decode<'r, R::Database>,
    String: Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let user = User::from_row(row)?;
        let password_hash = row.try_get("password")?;

        Ok(Self {
            user,
            password_hash,
        })
    }
}

pub struct UserRepository<DB: Database> {
    db: Pool<DB>,
    hash_cost: u32,
}

impl<DB: Database> Clone for UserRepository<DB> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            hash_cost: self.hash_cost,
        }
    }
}

impl<DB: Database> UserRepository<DB> {
    pub fn new(db: Pool<DB>, hash_cost: u32) -> UserRepository<DB> {
        UserRepository { db, hash_cost }
    }
}

impl<DB> UserRepository<DB>
where
    DB: Database,
    for<'a> <DB as sqlx::Database>::Arguments<'a>: IntoArguments<'a, DB>,
    for<'a> &'a Pool<DB>: Executor<'a, Database = DB>,

    for<'r> User: FromRow<'r, DB::Row>,

    for<'r> &'r str: ColumnIndex<DB::Row>,
    for<'r> String: Decode<'r, DB>,
    for<'r> String: Type<DB>,

    for<'e> &'e [u8]: Encode<'e, DB>,
    for<'e> &'e [u8]: Type<DB>,

    for<'e> i64: Encode<'e, DB>,
    i64: Type<DB>,

    for<'e> &'e str: Encode<'e, DB>,
    for<'e> &'e str: Type<DB>,
{
    pub async fn get(&self, id: Uuid) -> Result<User, UserError> {
        sqlx::query_as("SELECT * FROM user WHERE id = $1")
            .bind(id.into_bytes().as_slice())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(%error, "got sqlx error while fetching user");
                UserError::Sqlx(error)
            })?
            .ok_or(UserError::NotFound)
    }

    pub async fn authenticate(
        &self,
        data: UserData,
    ) -> Result<User, UserError> {
        let user: UserWithPassword = sqlx::query_as(
            "SELECT * FROM user WHERE username = $1",
        )
        .bind(data.username.as_str())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while fetching user");
            UserError::Sqlx(error)
        })?
        .ok_or(UserError::NotFound)?;

        let ok = verify_password(data.password, user.password_hash).await?;
        if !ok {
            return Err(UserError::PasswordMismatch);
        }

        Ok(user.user)
    }

    pub async fn create(
        &self,
        permission: Permission,
        data: UserData,
    ) -> Result<User, UserError> {
        let id = Uuid::new_v4();
        let now_ms = Utc::now().timestamp_millis();

        let password_hash =
            hash_password(self.hash_cost, data.password).await?;

        sqlx::query_as(
            "INSERT INTO user \
            (id, created_at, updated_at, permission, username, password) \
            VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(id.into_bytes().as_slice())
        .bind(now_ms)
        .bind(now_ms)
        .bind(permission.bits() as i64)
        .bind(data.username.as_str())
        .bind(password_hash.as_str())
        .fetch_one(&self.db)
        .await
        .map_err(|error| {
            if matches!(
                &error,
                sqlx::Error::Database(e) if e.is_unique_violation(),
            ) {
                return UserError::AlreadyExists(data.username);
            }

            tracing::error!(%error, "got sqlx error while creating user");
            UserError::Sqlx(error)
        })
    }

    pub async fn update_permission(
        &self,
        id: Uuid,
        permission: Permission,
    ) -> Result<User, UserError> {
        let now_ms = Utc::now().timestamp_millis();

        sqlx::query_as(
            "UPDATE user SET updated_at = $1, permission = $2 \
            WHERE id = $3 RETURNING *",
        )
        .bind(now_ms)
        .bind(permission.bits() as i64)
        .bind(id.into_bytes().as_slice())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while updating user");
            UserError::Sqlx(error)
        })?
        .ok_or(UserError::NotFound)
    }

    pub async fn update_password(
        &self,
        id: Uuid,
        password: String,
    ) -> Result<User, UserError> {
        let now_ms = Utc::now().timestamp_millis();

        let password_hash = hash_password(self.hash_cost, password).await?;

        sqlx::query_as(
            "UPDATE user SET updated_at = $1, password = $2 \
            WHERE id = $3 RETURNING *",
        )
        .bind(now_ms)
        .bind(password_hash.as_str())
        .bind(id.into_bytes().as_slice())
        .fetch_optional(&self.db)
        .await
        .map_err(|error| {
            tracing::error!(%error, "got sqlx error while updating user");
            UserError::Sqlx(error)
        })?
        .ok_or(UserError::NotFound)
    }

    pub async fn delete(&self, id: Uuid) -> Result<User, UserError> {
        sqlx::query_as("DELETE FROM user WHERE id = $1 RETURNING *")
            .bind(id.into_bytes().as_slice())
            .fetch_optional(&self.db)
            .await
            .map_err(|error| {
                tracing::error!(%error, "got sqlx error while deleting user");
                UserError::Sqlx(error)
            })?
            .ok_or(UserError::NotFound)
    }
}

async fn hash_password(
    cost: u32,
    password: String,
) -> Result<String, UserError> {
    spawn_blocking(move || bcrypt::hash(password, cost))
        .await
        .map_err(|error| {
            tracing::error!(
                %error,
                "got tokio error while handling bcrypt hash task",
            );
            UserError::BcryptHashFailed
        })?
        .map_err(|error| {
            tracing::error!(
                %error,
                "got bcrypt error while hashing password",
            );
            UserError::BcryptHashFailed
        })
}

async fn verify_password(
    password: String,
    hash: String,
) -> Result<bool, UserError> {
    spawn_blocking(move || bcrypt::verify(password, &hash))
        .await
        .map_err(|error| {
            tracing::error!(
                %error,
                "got tokio error while handling bcrypt verify task",
            );
            UserError::BcryptCompareFailed
        })?
        .map_err(|error| {
            tracing::error!(
                %error,
                "got bcrypt error while verifying password",
            );
            UserError::BcryptCompareFailed
        })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use sqlx::{migrate, Sqlite, SqlitePool};
    use test_log::test;
    use uuid::Uuid;

    use crate::{
        auth::Permission,
        user::{UserData, UserError},
    };

    use super::UserRepository;

    fn rand_string() -> String {
        Uuid::new_v4().to_string()
    }

    fn rand_data() -> UserData {
        UserData {
            username: rand_string(),
            password: rand_string(),
        }
    }

    async fn repository() -> UserRepository<Sqlite> {
        let db = SqlitePool::connect("sqlite::memory:").await.unwrap();
        migrate!().run(&db).await.unwrap();

        UserRepository::new(db, bcrypt::DEFAULT_COST)
    }

    #[test(tokio::test)]
    async fn test_create() {
        let repo = repository().await;

        let data = rand_data();
        let user = repo.create(Permission::ADMIN, data.clone()).await.unwrap();

        let fetched_user = repo
            .get(user.id)
            .await
            .expect("failed to fetch created user");

        assert_eq!(
            user, fetched_user,
            "fetched user mismatches the created one",
        );
    }

    #[test(tokio::test)]
    async fn test_authenticate() {
        let repo = repository().await;

        let data = rand_data();
        let user = repo.create(Permission::ADMIN, data.clone()).await.unwrap();

        let fetched_user = repo
            .authenticate(data.clone())
            .await
            .expect("failed to authenticate created user");
        assert_eq!(
            user, fetched_user,
            "fetched user mismatches the created one",
        );

        let mut data = data;
        data.password = rand_string();

        let res = repo.authenticate(data).await;
        assert!(
            matches!(res, Err(e) if matches!(e, UserError::PasswordMismatch)),
            "expected error while authenticating with different password",
        )
    }

    #[test(tokio::test)]
    async fn test_update_permission() {
        let repo = repository().await;

        let data = rand_data();
        let user = repo.create(Permission::ADMIN, data.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;

        let new_perm = Permission::UNPRIVILEGED.union(Permission::WRITE_USERS);
        let fetched_user =
            repo.update_permission(user.id, new_perm).await.unwrap();

        let mut old_user = user.clone();
        assert!(
            fetched_user.updated_at > old_user.updated_at,
            "updated_at field not changed",
        );

        old_user.permission = new_perm;
        old_user.updated_at = fetched_user.updated_at;

        assert_eq!(
            fetched_user, old_user,
            "updated user info differs from the intended one",
        );

        let fetched_user2 = repo.get(user.id).await.unwrap();
        assert_eq!(
            fetched_user2, old_user,
            "fetched user mismatches the updated one",
        );
    }

    #[test(tokio::test)]
    async fn test_update_password() {
        let repo = repository().await;

        let data = rand_data();
        let user = repo.create(Permission::ADMIN, data.clone()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;

        let new_passwd = rand_string();
        let fetched_user = repo
            .update_password(user.id, new_passwd.clone())
            .await
            .unwrap();

        let mut old_user = user.clone();
        assert!(
            fetched_user.updated_at > old_user.updated_at,
            "updated_at field not changed",
        );
        old_user.updated_at = fetched_user.updated_at;

        assert_eq!(
            fetched_user, old_user,
            "updated user info differs from the created one",
        );

        let res = repo.authenticate(data.clone()).await;
        assert!(
            matches!(res, Err(e) if matches!(e, UserError::PasswordMismatch)),
            "expected error while authenticating with different password",
        );

        let mut data = data;
        data.password = new_passwd;

        let fetched_user2 = repo
            .authenticate(data.clone())
            .await
            .expect("failed to authenticate after change password");

        assert_eq!(
            fetched_user2, old_user,
            "fetched user mismatches the updated one",
        );
    }

    #[test(tokio::test)]
    async fn test_delete() {
        let repo = repository().await;

        let res = repo.delete(Uuid::new_v4()).await;
        assert!(
            matches!(res, Err(UserError::NotFound)),
            "expected not found error while deleting non existent user",
        );

        let data = rand_data();
        let user = repo.create(Permission::ADMIN, data.clone()).await.unwrap();

        let fetched_user = repo.delete(user.id).await.unwrap();
        assert_eq!(
            fetched_user, fetched_user,
            "fetched data mismatches the created one",
        );

        let res = repo.get(user.id).await;
        assert!(
            matches!(res, Err(UserError::NotFound)),
            "expected not found error while fetching deleted user",
        );
    }
}
