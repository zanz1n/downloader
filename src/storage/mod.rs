use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{ColumnIndex, Decode, FromRow, Row, Type};
use uuid::Uuid;

pub mod manager;
pub mod repository;
pub mod routes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: ObjectData,
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

        let user_id: Vec<u8> = row.try_get("user_id")?;
        let user_id: [u8; 16] = user_id.try_into().map_err(|_| {
            sqlx::Error::Decode("parse `user_id` uuid out of range".into())
        })?;
        let user_id = Uuid::from_bytes(user_id);

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
            user_id,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectData {
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    #[serde(with = "hex_sha256")]
    pub checksum_256: [u8; 32],
}

mod hex_sha256 {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[inline]
    pub fn serialize<S: Serializer>(
        slice: &[u8],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        hex::encode(slice).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(deserializer)?;
        hex::decode(s)
            .map_err(|err| {
                serde::de::Error::custom(format!(
                    "failed to decode sha256 hex: {err}"
                ))
            })?
            .try_into()
            .map_err(|v: Vec<u8>| {
                serde::de::Error::custom(format!(
                    "the sha256 length is invalid: expected 32, got {}",
                    v.len()
                ))
            })
    }
}
