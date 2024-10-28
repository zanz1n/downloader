use std::{sync::Arc, time::Duration};

use axum::{extract::Path, routing, Extension, Router};
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

pub fn auth_routes<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .route("/self", routing::get(get_self))
        .route("/login", routing::post(post_login))
        .route("/signup", routing::post(post_signup))
        .route("/token/:id", routing::post(post_file_token))
        .route("/password", routing::put(update_self_password))
}

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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UpdatePasswordRequestData {
    pub username: String,
    pub old_password: String,
    pub new_password: String,
}

pub async fn get_self(
    Authorization(token): Authorization,
) -> Result<Json<Token>, DownloaderError> {
    Ok(Json(token))
}

pub async fn post_login(
    Extension(token_repo): Extension<Arc<TokenRepository>>,
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

pub async fn post_signup(
    Authorization(token): Authorization,
    Extension(token_repo): Extension<Arc<TokenRepository>>,
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Json(data): Json<LoginRequestData>,
) -> Result<Json<LoginResponseData>, DownloaderError> {
    if !token.can_write_users() {
        return Err(AuthError::AccessDenied.into());
    }

    let (data, permission) = data.split();
    let permission = permission.unwrap_or_else(|| match token {
        Token::Server => Permission::ADMIN,
        _ => Permission::UNPRIVILEGED,
    });

    let user = user_repo.create(permission, data).await?;
    let token = token_repo.generate_user_token(
        user.id,
        permission,
        user.username.clone(),
    )?;

    Ok(Json(LoginResponseData { user, token }))
}

pub async fn post_file_token(
    Authorization(token): Authorization,
    Extension(token_repo): Extension<Arc<TokenRepository>>,
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

pub async fn update_self_password(
    Extension(user_repo): Extension<UserRepository<Sqlite>>,
    Extension(token_repo): Extension<Arc<TokenRepository>>,
    Json(data): Json<UpdatePasswordRequestData>,
) -> Result<Json<LoginResponseData>, DownloaderError> {
    let mut user = user_repo
        .authenticate(UserData {
            username: data.username,
            password: data.old_password,
        })
        .await?;

    user = user_repo
        .update_password(user.id, data.new_password)
        .await?;

    let token = token_repo.generate_user_token(
        user.id,
        user.permission,
        user.username.clone(),
    )?;

    Ok(Json(LoginResponseData { user, token }))
}
