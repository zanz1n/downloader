use std::time::Duration;

use axum::{extract::Path, Extension};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use uuid::Uuid;

use crate::{
    errors::DownloaderError,
    storage::{repository::ObjectRepository, Object},
    user::{repository::UserRepository, User, UserData},
    utils::extractors::Json,
};

use super::{
    axum::Authorization, repository::TokenRepository, AuthError, Permission,
    Token,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequestData {
    pub username: String,
    pub password: String,
    pub permission: Option<Permission>,
}

impl LoginRequestData {
    #[inline]
    pub fn split(self) -> (UserData, Option<Permission>) {
        (
            UserData {
                password: self.password,
                username: self.username,
            },
            self.permission,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LoginResponseData {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileTokenRequestData {
    pub permission: Option<Permission>,
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileTokenResponseData {
    pub file: Object,
    pub token: String,
}

pub async fn login(
    Extension(token_repo): Extension<TokenRepository>,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Json(data): Json<LoginRequestData>,
) -> Result<Json<LoginResponseData>, DownloaderError> {
    let (data, permission) = data.split();
    let user = user_repo.authenticate(data).await?;

    let permission = if let Some(permission) = permission {
        if !user.permission.contains(permission) {
            return Err(AuthError::HigherPermissionRequired.into());
        }
        permission
    } else {
        user.permission
    };

    let token = token_repo.generate_user_token(
        user.id,
        permission,
        user.username.clone(),
    )?;

    Ok(Json(LoginResponseData { token, user }))
}

pub async fn signup(
    Authorization(token): Authorization,
    Extension(token_repo): Extension<TokenRepository>,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Json(data): Json<LoginRequestData>,
) -> Result<Json<LoginResponseData>, DownloaderError> {
    if !token.can_write_users() {
        return Err(AuthError::AccessDenied.into());
    }

    let (data, permission) = data.split();
    let permission = permission.unwrap_or(Permission::UNPRIVILEGED);

    let user = user_repo.create(permission, data).await?;
    let token = token_repo.generate_user_token(
        user.id,
        permission,
        user.username.clone(),
    )?;

    Ok(Json(LoginResponseData { user, token }))
}

pub async fn generate_file_token(
    Authorization(token): Authorization,
    Extension(token_repo): Extension<TokenRepository>,
    Extension(obj_repo): Extension<ObjectRepository<Sqlite>>,
    Path(id): Path<Uuid>,
    Json(data): Json<FileTokenRequestData>,
) -> Result<Json<FileTokenResponseData>, DownloaderError> {
    if !token.can_share() {
        return Err(AuthError::AccessDenied.into());
    }

    let permission = data.permission.unwrap_or(Permission::SINGLE_FILE_R);
    let duration = data
        .duration
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(3600));

    if !token.permission().contains(permission) {
        return Err(AuthError::HigherPermissionRequired.into());
    }

    let file = obj_repo.get(id).await?;

    let (can_access, issuer) = match &token {
        Token::User(user_token) => (
            token.can_write_all() || file.user_id == user_token.user_id,
            format!("user/{}", user_token.user_id),
        ),
        Token::File(file_token) => {
            tracing::warn!(
                file_id = %file_token.file_id,
                issuer = %file_token.issuer,
                "got a file token with `SHARE` permission"
            );
            return Err(AuthError::AccessDenied.into());
        }
        Token::Server => (true, "SRV".into()),
    };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let token = token_repo
        .generate_file_token(file.id, duration, issuer, permission)?;

    Ok(Json(FileTokenResponseData { file, token }))
}
