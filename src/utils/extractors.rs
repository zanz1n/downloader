use axum::{
    async_trait,
    extract::{FromRequest, FromRequestParts, Request},
    http::request::Parts,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use crate::errors::DownloaderError;

pub struct Query<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<S> for Query<T>
where
    T: for<'de> Deserialize<'de>,
    S: Send + Sync,
    T: Send + Sync,
{
    type Rejection = DownloaderError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        axum::extract::Query::from_request_parts(parts, state)
            .await
            .map(|v| Query(v.0))
            .map_err(|e| DownloaderError::Other(e.body_text(), e.status()))
    }
}

pub struct Json<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: for<'de> Deserialize<'de>,
    S: Send + Sync,
    T: Send + Sync,
{
    type Rejection = DownloaderError;

    async fn from_request(
        req: Request,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        axum::Json::from_request(req, state)
            .await
            .map(|v| Json(v.0))
            .map_err(|e| DownloaderError::Other(e.body_text(), e.status()))
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}
