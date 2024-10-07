use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Decode, FromRow, Row, Type};
use uuid::Uuid;

use crate::auth::Permission;

pub mod repository;

#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user not found")]
    NotFound,
    #[error("user with username `{0}` already exists")]
    AlreadyExists(String),
    #[error("incorrect password")]
    PasswordMismatch,
    #[error("bcrypt hash failed")]
    BcryptHashFailed,
    #[error("bcrypt compare failed")]
    BcryptCompareFailed,
    #[error("sqlx error: {0}")]
    Sqlx(sqlx::Error),
}

impl UserError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            UserError::NotFound => StatusCode::NOT_FOUND,
            UserError::AlreadyExists(..) => StatusCode::CONFLICT,
            UserError::PasswordMismatch => StatusCode::UNAUTHORIZED,
            UserError::BcryptHashFailed => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::BcryptCompareFailed => StatusCode::INTERNAL_SERVER_ERROR,
            UserError::Sqlx(..) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            UserError::NotFound => 1,
            UserError::AlreadyExists(..) => 2,
            UserError::PasswordMismatch => 3,
            UserError::BcryptHashFailed => 4,
            UserError::BcryptCompareFailed => 5,
            UserError::Sqlx(..) => 6,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub permission: Permission,
    pub username: String,
}

impl<'r, R: Row> FromRow<'r, R> for User
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

        let permission: i64 = row.try_get("permission")?;
        let permission: u8 = permission.try_into().map_err(|_| {
            sqlx::Error::Decode("parse `permission` u8 out of range".into())
        })?;
        let permission =
            Permission::from_bits(permission).ok_or_else(|| {
                sqlx::Error::Decode(
                    "parse `permission` invalid bitflags".into(),
                )
            })?;

        let username: String = row.try_get("username")?;

        Ok(Self {
            id,
            created_at,
            updated_at,
            permission,
            username,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
/// Struct contains sensitive information about user.
///
/// MUST NOT BE SERIALIZED!
pub struct UserData {
    pub username: String,
    pub password: String,
}
