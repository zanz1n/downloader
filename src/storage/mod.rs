use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod manager;
pub mod repository;
pub mod routes;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Object {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub data: ObjectData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
