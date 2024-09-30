use axum::{
    body::Body,
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
}

impl DownloaderError {
    #[inline]
    pub fn status_code(&self) -> StatusCode {
        match self {
            DownloaderError::Repository(e) => e.status_code(),
            DownloaderError::Object(e) => e.status_code(),
        }
    }

    pub fn custom_code(&self) -> u32 {
        let ic = match self {
            DownloaderError::Repository(e) => e.custom_code(),
            DownloaderError::Object(e) => e.custom_code(),
        };

        let c = match self {
            DownloaderError::Repository(..) => 1,
            DownloaderError::Object(..) => 2,
        };

        (c * 1000) + (ic as u32)
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
