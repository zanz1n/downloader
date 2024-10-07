use std::time::Duration;

use bitflags::bitflags;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod repository;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("failed to generate token")]
    GenerateTokenFailed,
    #[error("token expiration too long: got {got:?} while max is {max:?}")]
    TokenExpirationTooLong { got: Duration, max: Duration },

    #[error("the provided token is invalid")]
    InvalidToken,
    #[error("the provided token is expired")]
    ExpiredToken,
    #[error("the provided token must be used in the future")]
    ImatureToken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum Token {
    User(UserToken),
    File(FileToken),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserToken {
    // Jwt token information
    #[serde(rename = "sub")]
    pub user_id: Uuid,
    #[serde(rename = "iat", with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "exp", with = "chrono::serde::ts_seconds")]
    pub expiration: DateTime<Utc>,
    #[serde(rename = "iss")]
    pub issuer: String,

    // Custom information
    #[serde(rename = "perm")]
    pub permission: Permission,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileToken {
    // Jwt token information
    #[serde(rename = "sub")]
    pub file_id: Uuid,
    #[serde(rename = "iat", with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "exp", with = "chrono::serde::ts_seconds")]
    pub expiration: DateTime<Utc>,
    #[serde(rename = "iss")]
    pub issuer: String,

    // Custom information
    #[serde(rename = "perm")]
    pub permission: Permission,
}

impl Token {
    #[inline]
    pub fn permission(&self) -> Permission {
        match self {
            Token::User(p) => p.permission,
            Token::File(p) => p.permission,
        }
    }

    #[inline]
    pub fn can_read_owned(&self) -> bool {
        true
    }

    #[inline]
    pub fn can_share(&self) -> bool {
        self.permission().contains(Permission::SHARE)
    }

    #[inline]
    pub fn can_read_all(&self) -> bool {
        self.permission().contains(Permission::READ_ALL)
    }

    #[inline]
    pub fn can_write_owned(&self) -> bool {
        let perm = self.permission();
        perm.contains(Permission::WRITE_OWNED)
            || perm.contains(Permission::WRITE_ALL)
    }

    #[inline]
    pub fn can_write_all(&self) -> bool {
        self.permission().contains(Permission::WRITE_ALL)
    }

    #[inline]
    pub fn can_read_users(&self) -> bool {
        self.permission().contains(Permission::READ_USERS)
    }

    #[inline]
    pub fn can_write_users(&self) -> bool {
        self.permission().contains(Permission::WRITE_USERS)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Permission: u8 {
        const SHARE = 1;

        const WRITE_OWNED = 1 << 1;

        const READ_ALL = 1 << 2;
        const WRITE_ALL = 1 << 3;

        const READ_USERS = 1 << 4;
        const WRITE_USERS = 1 << 5;

        const ADMIN = Self::SHARE.bits()
        | Self::WRITE_OWNED.bits()
        | Self::READ_ALL.bits()
        | Self::WRITE_ALL.bits()
        | Self::READ_USERS.bits()
        | Self::WRITE_USERS.bits();

        const UNPRIVILEGED = Self::SHARE.bits()
        | Self::WRITE_OWNED.bits()
        | Self::READ_USERS.bits();

        const SINGLE_FILE_R = 0;
        const SINGLE_FILE_RW = Self::WRITE_OWNED.bits();
    }
}
