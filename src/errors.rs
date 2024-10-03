use axum::{
    body::Body,
    extract::multipart::MultipartError,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::storage::{manager::ObjectError, repository::RepositoryError};

#[derive(Debug, thiserror::Error)]
pub enum DownloaderError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Object(#[from] ObjectError),
    #[error(transparent)]
    Http(#[from] HttpError),

    #[error(transparent)]
    AxumHttp(#[from] axum::http::Error),
    #[error(transparent)]
    Multipart(#[from] MultipartError),
}

impl DownloaderError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            DownloaderError::Repository(e) => e.status_code(),
            DownloaderError::Object(e) => e.status_code(),
            DownloaderError::Http(e) => e.status_code(),
            DownloaderError::AxumHttp(..) => StatusCode::INTERNAL_SERVER_ERROR,
            DownloaderError::Multipart(e) => e.status(),
        }
    }

    pub fn custom_code(&self) -> u32 {
        let ic = match self {
            DownloaderError::Repository(e) => e.custom_code(),
            DownloaderError::Object(e) => e.custom_code(),
            DownloaderError::Http(e) => e.custom_code(),
            DownloaderError::AxumHttp(..) => 0,
            DownloaderError::Multipart(..) => 0,
        };

        let c = match self {
            DownloaderError::Repository(..) => 1,
            DownloaderError::Object(..) => 2,
            DownloaderError::Http(..) => 3,
            DownloaderError::AxumHttp(..) => 100,
            DownloaderError::Multipart(..) => 101,
        };

        (c * 1000) + (ic as u32)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error(
        "the provided multipart form length is invalid: \
        expected {expected}, got {got}"
    )]
    InvalidFormLength { expected: usize, got: usize },
    #[error("the provided form boundary is invalid")]
    InvalidFormBoundary,
}

impl HttpError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            HttpError::InvalidFormBoundary => StatusCode::BAD_REQUEST,
            HttpError::InvalidFormLength { .. } => StatusCode::BAD_REQUEST,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            HttpError::InvalidFormLength { .. } => todo!(),
            HttpError::InvalidFormBoundary => 1,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
    error_code: u32,
}

impl IntoResponse for DownloaderError {
    fn into_response(self) -> Response {
        let mut mime_type = mime::APPLICATION_JSON.essence_str();

        let body_data = serde_json::to_string(&ErrorResponse {
            error: self.to_string(),
            error_code: self.custom_code(),
        })
        .unwrap_or_else(|err| {
            mime_type = mime::TEXT_PLAIN.essence_str();
            err.to_string()
        });

        Response::builder()
            .header(header::CONTENT_TYPE, mime_type)
            .status(self.status_code())
            .body(Body::new(body_data))
            .expect("failed to build response")
    }
}
