use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::{multipart::MultipartError, Multipart, Path, Request},
    http::{header, HeaderValue},
    response::Response,
    routing, Extension, Router,
};
use bytes::Bytes;
use futures_util::{Stream, TryStreamExt};
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use tokio_util::io::ReaderStream;
use tracing::Instrument;
use uuid::Uuid;

use crate::{
    auth::{axum::Authorization, AuthError, Token},
    errors::{DownloaderError, HttpError},
    storage::ObjectData,
    utils::extractors::{Json, Query},
};

use super::{
    manager::{Manager, ObjectManager},
    repository::ObjectRepository,
    Object,
};

pub fn file_routes<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router
        .route("/", routing::get(get_all_files))
        .route("/user/:user_id", routing::get(get_files_by_user))
        .route("/:id", routing::get(get_file))
        .route("/:id/data", routing::get(download_file))
        .route("/", routing::post(upload_file))
        .route("/multipart", routing::post(upload_file_multipart))
        .route("/:id", routing::put(update_file))
        .route("/:id/data", routing::put(update_file_data))
        .route("/:id/multipart", routing::put(update_file_data_multipart))
        .route("/:id", routing::delete(delete_file))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostFileRequestData {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PaginationData {
    #[serde(default = "default_pagination_limit")]
    pub limit: u32,
    #[serde(default = "default_pagination_offset")]
    pub offset: u32,
}

const fn default_pagination_limit() -> u32 {
    100
}

const fn default_pagination_offset() -> u32 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateFileRequestData {
    pub name: String,
    pub mime_type: String,
}

pub async fn get_all_files(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Query(data): Query<PaginationData>,
) -> Result<Json<Vec<Object>>, DownloaderError> {
    if !token.can_read_all() {
        return Err(AuthError::AccessDenied.into());
    }

    repo.get_all(data.limit, data.offset)
        .await
        .map(Json)
        .map_err(DownloaderError::Repository)
}

pub async fn get_files_by_user(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Path(user_id): Path<Uuid>,
    Query(data): Query<PaginationData>,
) -> Result<Json<Vec<Object>>, DownloaderError> {
    let can_access = token.can_read_all()
        || match token {
            Token::User(user_token) => user_token.user_id == user_id,
            _ => false,
        };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    repo.get_by_user(user_id, data.limit, data.offset)
        .await
        .map(Json)
        .map_err(DownloaderError::Repository)
}

pub async fn get_file(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>, DownloaderError> {
    let object = repo.get(id).await?;

    let can_access = token.can_read_all()
        || (object.user_id
            == match token {
                Token::User(user_token) => user_token.user_id,
                _ => Uuid::nil(),
            });

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    Ok(Json(object))
}

pub async fn download_file(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
) -> Result<Response, DownloaderError> {
    let object = repo.get(id).await?;

    let can_access = token.can_read_all()
        || (object.user_id
            == match token {
                Token::User(user_token) => user_token.user_id,
                _ => Uuid::nil(),
            });

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let reader = manager.fetch(id).await?;

    Response::builder()
        .header(header::CONTENT_TYPE, object.data.mime_type)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", object.data.name),
        )
        .header(header::CONTENT_LENGTH, object.data.size.to_string())
        .body(Body::from_stream(ReaderStream::new(reader)))
        .map_err(DownloaderError::from)
}

pub async fn upload_file(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Query(PostFileRequestData { name }): Query<PostFileRequestData>,
    req: Request,
) -> Result<Json<Object>, DownloaderError> {
    let (stream, mime_type) = extract_request_body_file(req);

    post_file_internal(token, repo, manager, stream, name, mime_type)
        .await
        .map(Json)
}

pub async fn upload_file_multipart(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    mut multipart: Multipart,
) -> Result<Json<Object>, DownloaderError> {
    let (stream, name, mime_type) =
        extract_multipart_file(&mut multipart).await?;

    post_file_internal(token, repo, manager, stream, name, mime_type)
        .await
        .map(Json)
}

pub async fn update_file(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Path(id): Path<Uuid>,
    Json(data): Json<UpdateFileRequestData>,
) -> Result<Json<Object>, DownloaderError> {
    // Placed before to avoid unecessary database queries in case the
    // write permission is missing
    if !token.can_write_owned() {
        return Err(AuthError::AccessDenied.into());
    }

    let can_access = match &token {
        Token::User(user_token) => {
            let obj = repo.get(id).await?;

            obj.user_id == user_token.user_id || token.can_write_all()
        }
        Token::File(file_token) => file_token.file_id == id,
        Token::Server => true,
    };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let obj = repo.update_info(id, data.name, data.mime_type).await?;
    Ok(Json(obj))
}

pub async fn update_file_data(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
    Query(PostFileRequestData { name }): Query<PostFileRequestData>,
    req: Request,
) -> Result<Json<Object>, DownloaderError> {
    let (stream, mime_type) = extract_request_body_file(req);
    // pin_mut!(reader);

    update_file_internal(token, repo, manager, id, stream, name, mime_type)
        .await
        .map(Json)
}

pub async fn update_file_data_multipart(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Object>, DownloaderError> {
    let (stream, name, mime_type) =
        extract_multipart_file(&mut multipart).await?;
    // pin_mut!(reader);

    update_file_internal(token, repo, manager, id, stream, name, mime_type)
        .await
        .map(Json)
}

pub async fn delete_file(
    Authorization(token): Authorization,
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>, DownloaderError> {
    // Placed before to avoid unecessary database queries in case the
    // write permission is missing
    if !token.can_write_owned() {
        return Err(AuthError::AccessDenied.into());
    }

    let can_access = match &token {
        Token::User(user_token) => {
            let obj = repo.get(id).await?;

            obj.user_id == user_token.user_id || token.can_write_all()
        }
        Token::File(file_token) => file_token.file_id == id,
        Token::Server => true,
    };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let obj = repo.delete(id).await?;

    tokio::spawn(async move {
        manager
            .delete(id)
            .instrument(tracing::span!(
                tracing::Level::WARN,
                "delete_background"
            ))
            .await
    });

    Ok(Json(obj))
}

async fn extract_multipart_file<'a>(
    multipart: &'a mut Multipart,
) -> Result<
    (
        futures_util::stream::MapErr<
            axum::extract::multipart::Field<'a>,
            impl FnMut(MultipartError) -> io::Error,
        >,
        String,
        String,
    ),
    DownloaderError,
> {
    let field =
        multipart
            .next_field()
            .await?
            .ok_or(HttpError::InvalidFormLength {
                expected: 1,
                got: 0,
            })?;

    let name = field
        .file_name()
        .ok_or(HttpError::InvalidFormBoundary)?
        .to_string();

    let mime_type = field
        .content_type()
        .ok_or(HttpError::InvalidFormBoundary)?
        .to_string();

    let field_stream =
        field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));

    Ok((field_stream, name, mime_type))
}

fn extract_request_body_file(
    req: Request,
) -> (
    futures_util::stream::MapErr<
        axum::body::BodyDataStream,
        impl FnMut(axum::Error) -> io::Error,
    >,
    String,
) {
    let mime_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .unwrap_or(&HeaderValue::from_static(mime::OCTET_STREAM.as_str()))
        .to_str()
        .unwrap_or(mime::OCTET_STREAM.as_str())
        .to_string();

    let stream = req.into_body().into_data_stream();
    let stream =
        stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));

    (stream, mime_type)
}

async fn post_file_internal(
    token: Token,
    repo: ObjectRepository<Sqlite>,
    manager: Arc<ObjectManager>,
    stream: impl Stream<Item = Result<Bytes, io::Error>> + Unpin + Send,
    name: String,
    mime_type: String,
) -> Result<Object, DownloaderError> {
    if !token.can_write_owned() {
        return Err(AuthError::AccessDenied.into());
    }
    let token = match token {
        Token::User(user_token) => user_token,
        _ => return Err(AuthError::AccessDenied.into()),
    };

    let id = Uuid::new_v4();
    let (size, checksum_256) = manager.store(id, stream).await?;

    let data = ObjectData {
        name,
        mime_type,
        size,
        checksum_256,
    };

    match repo.create(id, token.user_id, data).await {
        Ok(v) => Ok(v),
        Err(error) => {
            tracing::error!(
                target: "routes::post",
                %error,
                %id,
                "create object entry failed after store",
            );

            let _ = manager.delete(id).await.map_err(|error| {
                tracing::error!(
                    target: "storage::routes::post",
                    %error,
                    %id,
                    "delete object without repository entry failed",
                );
            });

            Err(error.into())
        }
    }
}

async fn update_file_internal(
    token: Token,
    repo: ObjectRepository<Sqlite>,
    manager: Arc<ObjectManager>,
    id: Uuid,
    stream: impl Stream<Item = Result<Bytes, io::Error>> + Unpin + Send,
    name: String,
    mime_type: String,
) -> Result<Object, DownloaderError> {
    // Placed before to avoid unecessary database queries in case the
    // write permission is missing
    if !token.can_write_owned() {
        return Err(AuthError::AccessDenied.into());
    }

    let can_access = match &token {
        Token::User(user_token) => {
            let obj = repo.get(id).await?;

            obj.user_id == user_token.user_id || token.can_write_all()
        }
        Token::File(file_token) => file_token.file_id == id,
        Token::Server => true,
    };

    if !can_access {
        return Err(AuthError::AccessDenied.into());
    }

    let (size, checksum_256) = manager.store(id, stream).await?;

    repo.update(
        id,
        ObjectData {
            name,
            mime_type,
            size,
            checksum_256,
        },
    )
    .await
    .map_err(|error| {
        tracing::error!(
            target: "storage::routes::update",
            %error,
            %id,
            "update object entry failed after store",
        );
        error.into()
    })
}
