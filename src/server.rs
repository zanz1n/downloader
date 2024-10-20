use std::{fmt::Display, iter::once, time::Duration};

use axum::{
    body::Body,
    handler::Handler,
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    routing, Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::{CatchPanicLayer, ResponseForPanic},
    cors::CorsLayer,
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
    let path = req.uri().path().trim_start_matches("/");

    if path.starts_with("api") {
        return DownloaderError::Http(HttpError::RouteNotFound).into_response();
    }

    tracing::debug!(
        path = %req.uri().path(),
        version = ?req.version(),
        "fetch static resource",
    );

    match Asset::get(path) {
        Some(content) => (
            axum::http::StatusCode::OK,
            [(
                header::CONTENT_TYPE,
                content.metadata.mimetype().to_string(),
            )],
            content.data,
        ),
        None => {
            if path.starts_with("_app") {
                return DownloaderError::Http(HttpError::ResourceNotFound)
                    .into_response();
            } else {
                Asset::get("index.html")
                    .map(|content| {
                        (
                            axum::http::StatusCode::OK,
                            [(
                                header::CONTENT_TYPE,
                                mime::TEXT_HTML.as_ref().to_owned(),
                            )],
                            content.data,
                        )
                    })
                    .unwrap_or_else(|| {
                        (
                            axum::http::StatusCode::NOT_FOUND,
                            [(
                                header::CONTENT_TYPE,
                                mime::TEXT_PLAIN.as_ref().to_owned(),
                            )],
                            std::borrow::Cow::Borrowed(b"Not Found"),
                        )
                    })
            }
        }
    }
    .into_response()
}

pub fn layer_root_router<S>(router: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let layer = ServiceBuilder::new()
        .layer(SetSensitiveHeadersLayer::new(once(header::AUTHORIZATION)))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(CustomMakeSpan)
                .on_response(CustomOnResponse)
                .on_request(CustomOnRequest)
                .on_failure(CustomOnFailure),
        )
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static("axum/0.7.5"),
        ))
        .layer(CatchPanicLayer::custom(JsonPanicHandler))
        .layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash());

    let fallback_layer = ServiceBuilder::new()
        .layer(SetSensitiveHeadersLayer::new(once(header::AUTHORIZATION)))
        .layer(SetResponseHeaderLayer::overriding(
            header::SERVER,
            HeaderValue::from_static("axum/0.7.5"),
        ))
        .layer(CatchPanicLayer::new())
        .layer(CorsLayer::permissive())
        .layer(NormalizePathLayer::trim_trailing_slash());

    router
        .layer(layer)
        .fallback(routing::any(fallback_handler.layer(fallback_layer)))
}
