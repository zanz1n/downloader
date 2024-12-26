use std::{future::Future, io};

use axum::http::StatusCode;
use bytes::Bytes;
use futures_util::Stream;
use tokio::io::AsyncRead;
use uuid::Uuid;

#[cfg(any(not(feature = "io-uring"), test))]
mod stdfs;
#[cfg(test)]
mod test_utils;

#[cfg(not(feature = "io-uring"))]
pub use stdfs::SyncFsManager as ObjectManager;

#[derive(Debug, thiserror::Error)]
pub enum ObjectError {
    #[error("io error in file system: {0}")]
    IoError(#[from] io::Error),
    #[error("file not found")]
    NotFound,
}

impl ObjectError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            ObjectError::IoError(..) => StatusCode::INTERNAL_SERVER_ERROR,
            ObjectError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            ObjectError::IoError(..) => 1,
            ObjectError::NotFound => 2,
        }
    }
}

pub trait Manager {
    fn store(
        &self,
        id: Uuid,
        stream: impl Stream<Item = Result<Bytes, io::Error>> + Unpin + Send,
    ) -> impl Future<Output = Result<(u64, [u8; 32]), ObjectError>> + Send;

    fn fetch(
        &self,
        id: Uuid,
    ) -> impl Future<
        Output = Result<impl AsyncRead + Unpin + Send + 'static, ObjectError>,
    > + Send;

    fn delete(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<(), ObjectError>> + Send;
}
