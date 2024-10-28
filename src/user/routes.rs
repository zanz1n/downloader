use axum::{extract::Path, routing, Extension, Router};
use serde::Deserialize;
use sqlx::Sqlite;
use uuid::Uuid;

use crate::{
    auth::{axum::Authorization, AuthError, Permission, Token},
    errors::DownloaderError,
    utils::extractors::Json,
};

use super::{repository::UserRepository, User};

pub fn user_routes<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .route("/self", routing::get(get_self))
        .route("/:id", routing::get(get_user))
        .route("/:id/password", routing::put(update_user_password))
        .route("/:id/permission", routing::put(update_user_permission))
        .route("/self", routing::delete(delete_self))
        .route("/:id", routing::delete(delete_user))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdatePasswordRequestData {
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdatePermissionRequestData {
    pub permission: Permission,
}

pub async fn get_self(
    Authorization(token): Authorization,
    ext: Extension<UserRepository<Sqlite>>,
) -> Result<Json<User>, DownloaderError> {
    let id = match token {
        Token::User(user_token) => user_token.user_id,
        _ => return Err(AuthError::AccessDenied.into()),
    };

    get_user(Authorization(Token::Server), ext, Path(id)).await
}

pub async fn get_user(
    Authorization(token): Authorization,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, DownloaderError> {
    let can_access = match &token {
        Token::User(user_token) => {
            user_token.user_id == id || token.can_read_users()
        }
        Token::File(_) => token.can_read_users(),
        Token::Server => true,
    };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let user = user_repo.get(id).await?;
    Ok(Json(user))
}

pub async fn update_user_password(
    Authorization(token): Authorization,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePasswordRequestData>,
) -> Result<Json<User>, DownloaderError> {
    if !token.can_write_users() {
        return Err(AuthError::AccessDenied.into());
    }

    let user = user_repo.update_password(id, data.password).await?;
    Ok(Json(user))
}

pub async fn update_user_permission(
    Authorization(token): Authorization,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdatePermissionRequestData>,
) -> Result<Json<User>, DownloaderError> {
    if !token.can_write_users() {
        return Err(AuthError::AccessDenied.into());
    }

    let user = user_repo.update_permission(id, data.permission).await?;
    Ok(Json(user))
}

pub async fn delete_self(
    Authorization(token): Authorization,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
) -> Result<Json<User>, DownloaderError> {
    let id = match token {
        Token::User(user_token) => user_token.user_id,
        _ => return Err(AuthError::AccessDenied.into()),
    };

    let user = user_repo.delete(id).await?;
    Ok(Json(user))
}

pub async fn delete_user(
    Authorization(token): Authorization,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, DownloaderError> {
    if !token.can_write_users() {
        return Err(AuthError::AccessDenied.into());
    }

    let user = user_repo.delete(id).await?;
    Ok(Json(user))
}
