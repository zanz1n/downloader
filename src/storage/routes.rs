use std::{io, sync::Arc};

use axum::{
    body::Body,
    extract::{multipart::MultipartError, Multipart, Path, Query, Request},
    http::{header, HeaderValue},
    response::Response,
    Extension, Json,
};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::Sqlite;
use tokio::io::AsyncRead;
use tokio_util::io::{ReaderStream, StreamReader};
use tracing::Instrument;
use uuid::Uuid;

use crate::{
    errors::{DownloaderError, HttpError},
    storage::ObjectData,
};

use super::{manager::ObjectManager, repository::ObjectRepository, Object};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostFileData {
    pub name: String,
}

pub async fn get_file(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
) -> Result<Response, DownloaderError> {
    let object = repo.get(id).await?;
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

pub async fn post_file(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Query(PostFileData { name }): Query<PostFileData>,
    req: Request,
) -> Result<Json<Object>, DownloaderError> {
    let (reader, mime_type) = extract_request_body_file(req).await;
    // pin_mut!(reader);

    post_file_internal(repo, manager, reader, name, mime_type)
        .await
        .map(Json)
}

pub async fn post_file_multipart(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    mut multipart: Multipart,
) -> Result<Json<Object>, DownloaderError> {
    let (reader, name, mime_type) =
        extract_multipart_file(&mut multipart).await?;
    // pin_mut!(reader);

    post_file_internal(repo, manager, reader, name, mime_type)
        .await
        .map(Json)
}

pub async fn delete_file(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Object>, DownloaderError> {
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

pub async fn update_file(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
    Query(PostFileData { name }): Query<PostFileData>,
    req: Request,
) -> Result<Json<Object>, DownloaderError> {
    let (reader, mime_type) = extract_request_body_file(req).await;
    // pin_mut!(reader);

    update_file_internal(repo, manager, id, reader, name, mime_type)
        .await
        .map(Json)
}

pub async fn update_file_multipart(
    Extension(repo): Extension<ObjectRepository<Sqlite>>,
    Extension(manager): Extension<Arc<ObjectManager>>,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<Object>, DownloaderError> {
    let (reader, name, mime_type) =
        extract_multipart_file(&mut multipart).await?;
    // pin_mut!(reader);

    update_file_internal(repo, manager, id, reader, name, mime_type)
        .await
        .map(Json)
}

async fn extract_multipart_file<'a>(
    multipart: &'a mut Multipart,
) -> Result<
    (
        StreamReader<
            futures_util::stream::MapErr<
                axum::extract::multipart::Field<'a>,
                impl FnMut(MultipartError) -> io::Error,
            >,
            axum::body::Bytes,
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

    Ok((StreamReader::new(field_stream), name, mime_type))
}

async fn extract_request_body_file(
    req: Request,
) -> (
    StreamReader<
        futures_util::stream::MapErr<
            axum::body::BodyDataStream,
            impl FnMut(axum::Error) -> io::Error,
        >,
        axum::body::Bytes,
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

    let reader = StreamReader::new(stream);

    (reader, mime_type)
}

async fn post_file_internal(
    repo: ObjectRepository<Sqlite>,
    manager: Arc<ObjectManager>,
    reader: impl AsyncRead + Unpin,
    name: String,
    mime_type: String,
) -> Result<Object, DownloaderError> {
    let id = Uuid::new_v4();
    let (size, checksum_256) = manager.store(id, reader).await?;

    repo.create(
        id,
        ObjectData {
            name,
            mime_type,
            size,
            checksum_256,
        },
    )
    .await
    .map_err(DownloaderError::Repository)
}

async fn update_file_internal(
    repo: ObjectRepository<Sqlite>,
    manager: Arc<ObjectManager>,
    id: Uuid,
    reader: impl AsyncRead + Unpin,
    name: String,
    mime_type: String,
) -> Result<Object, DownloaderError> {
    let (size, checksum_256) = manager.store(id, reader).await?;

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
    .map_err(DownloaderError::Repository)
}
