use std::{fmt::Display, iter::once, time::Duration};

use axum::{
    body::Body,
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    routing, Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::{CatchPanicLayer, ResponseForPanic},
    cors::CorsLayer,
    decompression::RequestDecompressionLayer,
    normalize_path::NormalizePathLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    set_header::SetResponseHeaderLayer,
    trace::{MakeSpan, OnFailure, OnRequest, OnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    errors::{DownloaderError, HttpError},
    utils::fmt::fmt_duration,
};

#[cfg(feature = "embed")]
#[derive(rust_embed::Embed)]
#[folder = "frontend/build"]
pub struct Asset;

#[derive(Clone)]
struct CustomOnResponse;

impl<B> OnResponse<B> for CustomOnResponse {
    #[inline]
    fn on_response(
        self,
        response: &axum::http::Response<B>,
        latency: Duration,
        span: &tracing::Span,
    ) {
        let _guard = span.enter();
        let latency = fmt_duration(latency);

        tracing::info!(
            target: "http_logs",
            %latency,
            status = ?response.status(),
            version = ?response.version(),
            "finished processing request",
        );
    }
}

#[derive(Clone)]
struct CustomOnRequest;

impl<B> OnRequest<B> for CustomOnRequest {
    #[inline]
    fn on_request(
        &mut self,
        _request: &axum::http::Request<B>,
        span: &tracing::Span,
    ) {
        let _guard = span.enter();

        tracing::info!(
            target: "http_logs",
            "started processing request",
        );
    }
}

#[derive(Clone)]
struct CustomMakeSpan;

impl<B> MakeSpan<B> for CustomMakeSpan {
    #[inline]
    fn make_span(&mut self, request: &axum::http::Request<B>) -> tracing::Span {
        tracing::span!(
            Level::INFO,
            "request",
            method = %request.method().as_str(),
            path = %request.uri().path(),
            version = ?request.version(),
        )
    }
}

#[derive(Clone)]
struct CustomOnFailure;

impl<C: Display> OnFailure<C> for CustomOnFailure {
    #[inline]
    fn on_failure(
        &mut self,
        failure_classification: C,
        latency: Duration,
        span: &tracing::Span,
    ) {
        let _guard = span.enter();
        let latency = fmt_duration(latency);

        tracing::error!(
            target: "http_logs",
            classification = failure_classification.to_string(),
            %latency,
            "failure while processing request",
        );
    }
}

#[derive(Debug, Clone)]
struct JsonPanicHandler;

impl ResponseForPanic for JsonPanicHandler {
    type ResponseBody = Body;

    fn response_for_panic(
        &mut self,
        err: Box<dyn std::any::Any + Send + 'static>,
    ) -> axum::http::Response<Self::ResponseBody> {
        if let Some(s) = err.downcast_ref::<String>() {
            tracing::error!(target: "http_logs", "service panicked: {}", s);
        } else if let Some(s) = err.downcast_ref::<&str>() {
            tracing::error!(target: "http_logs", "service panicked: {}", s);
        } else {
            tracing::error!(
                target: "http_logs",
                "service panicked but `CatchPanic` was unable to downcast the panic info"
            );
        };

        DownloaderError::Http(HttpError::ServicePanicked).into_response()
    }
}

#[cfg(not(feature = "embed"))]
async fn fallback_handler() -> Response {
    DownloaderError::Http(HttpError::RouteNotFound).into_response()
}

#[cfg(feature = "embed")]
async fn fallback_handler(req: axum::extract::Request) -> Response {
    use std::borrow::Cow;

    use axum::http::StatusCode;

    const NO_CACHE_HEADER: &'static str =
        "no-cache, no-store, max-age=0, must-revalidate";
    const CACHE_HEADER: &'static str = "public, max-age=31536000";

    const NOT_FOUND_STATUS: (
        StatusCode,
        Cow<'static, str>,
        &'static str,
        Cow<'static, [u8]>,
    ) = (
        StatusCode::NOT_FOUND,
        Cow::Borrowed("text/plain"),
        NO_CACHE_HEADER,
        Cow::Borrowed(b"Not Found".as_slice()),
    );

    let path = req.uri().path().trim_start_matches("/");

    if path.starts_with("api") {
        return DownloaderError::Http(HttpError::RouteNotFound).into_response();
    }

    tracing::debug!(
        path = %req.uri().path(),
        version = ?req.version(),
        "fetch static resource",
    );

    let (status, content_type, cache_control, data) = match Asset::get(path) {
        Some(content) => (
            StatusCode::OK,
            Cow::Owned(content.metadata.mimetype().to_owned()),
            CACHE_HEADER,
            content.data,
        ),
        None => {
            if path.starts_with("_app") {
                NOT_FOUND_STATUS
            } else {
                Asset::get("index.html")
                    .map(|content| {
                        (
                            StatusCode::OK,
                            Cow::Owned(content.metadata.mimetype().to_owned()),
                            NO_CACHE_HEADER,
                            content.data,
                        )
                    })
                    .unwrap_or_else(|| NOT_FOUND_STATUS)
            }
        }
    };

    let content_type = match content_type {
        Cow::Borrowed(s) => HeaderValue::from_static(s),
        Cow::Owned(s) => HeaderValue::from_str(&s).unwrap(),
    };

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_control)
        .body(Body::from(data))
        .unwrap()
}

pub fn layer_root_router<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let layer = ServiceBuilder::new()
        .layer(SetSensitiveHeadersLayer::new(once(header::AUTHORIZATION)))
        .layer(RequestDecompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan)
                .on_response(CustomOnResponse)
                .on_request(CustomOnRequest)
                .on_failure(CustomOnFailure),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static("axum/0.7"),
        ))
        .layer(CatchPanicLayer::custom(JsonPanicHandler))
        .layer(CorsLayer::permissive().max_age(Duration::from_secs(86400)))
        .layer(NormalizePathLayer::trim_trailing_slash());

    #[cfg(feature = "embed")]
    {
        use axum::handler::Handler;
        use tower_http::compression::CompressionLayer;

        let fallback_layer = ServiceBuilder::new()
            .layer(SetSensitiveHeadersLayer::new(once(header::AUTHORIZATION)))
            .layer(SetResponseHeaderLayer::overriding(
                header::SERVER,
                HeaderValue::from_static("axum/0.7"),
            ))
            .layer(CatchPanicLayer::new())
            .layer(RequestDecompressionLayer::new())
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive().max_age(Duration::from_secs(86400)))
            .layer(NormalizePathLayer::trim_trailing_slash());

        return router
            .layer(layer)
            .fallback(routing::any(fallback_handler.layer(fallback_layer)));
    }

    #[cfg(not(feature = "embed"))]
    {
        return router.fallback(routing::any(fallback_handler)).layer(layer);
    }
}
