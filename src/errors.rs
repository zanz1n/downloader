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
    #[error("Repository error: {0}")]
    Repository(#[from] RepositoryError),
    #[error("Storage error: {0}")]
    Object(#[from] ObjectError),
    #[error("Http error: {0}")]
    Http(#[from] HttpError),

    #[error("Http error: {0}")]
    AxumHttp(#[from] axum::http::Error),
    #[error("Multipart form error: {0}")]
    Multipart(#[from] MultipartError),

    #[error("{0}")]
    Other(String, StatusCode),
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
            DownloaderError::Other(.., code) => *code,
        }
    }

    pub fn custom_code(&self) -> u32 {
        let ic = match self {
            DownloaderError::Repository(e) => e.custom_code(),
            DownloaderError::Object(e) => e.custom_code(),
            DownloaderError::Http(e) => e.custom_code(),
            DownloaderError::AxumHttp(..) => 0,
            DownloaderError::Multipart(..) => 0,
            DownloaderError::Other(..) => 0,
        };

        let c = match self {
            DownloaderError::Repository(..) => 1,
            DownloaderError::Object(..) => 2,
            DownloaderError::Http(..) => 3,
            DownloaderError::AxumHttp(..) => 100,
            DownloaderError::Multipart(..) => 101,
            DownloaderError::Other(..) => 0,
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
    #[error("route not found")]
    RouteNotFound,
    #[error("service panicked")]
    ServicePanicked,
}

impl HttpError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            HttpError::InvalidFormBoundary => StatusCode::BAD_REQUEST,
            HttpError::InvalidFormLength { .. } => StatusCode::BAD_REQUEST,
            HttpError::RouteNotFound => StatusCode::NOT_FOUND,
            HttpError::ServicePanicked => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[inline]
    pub fn custom_code(&self) -> u8 {
        match self {
            HttpError::InvalidFormLength { .. } => 1,
            HttpError::InvalidFormBoundary => 2,
            HttpError::RouteNotFound => 100,
            HttpError::ServicePanicked => 255,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub error_code: u32,
    #[serde(skip_serializing)]
    pub status_code: StatusCode,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let mut mime_type = mime::APPLICATION_JSON.essence_str();

        let body_data = serde_json::to_string(&self).unwrap_or_else(|err| {
            mime_type = mime::TEXT_PLAIN.essence_str();
            err.to_string()
        });

        Response::builder()
            .header(header::CONTENT_TYPE, mime_type)
            .status(self.status_code)
            .body(Body::new(body_data))
            .expect("failed to build response")
    }
}

impl IntoResponse for DownloaderError {
    #[inline]
    fn into_response(self) -> Response {
        ErrorResponse {
            error: self.to_string(),
            error_code: self.custom_code(),
            status_code: self.status_code(),
        }
        .into_response()
    }
}
