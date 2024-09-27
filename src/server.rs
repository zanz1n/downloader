use std::{fmt::Display, iter::once, time::Duration};

use axum::{
    http::{header, HeaderValue},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    decompression::RequestDecompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    set_header::SetResponseHeaderLayer,
    timeout::TimeoutLayer,
    trace::{MakeSpan, OnFailure, OnRequest, OnResponse, TraceLayer},
};
use tracing::Level;

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
            Level::DEBUG,
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

#[inline]
fn fmt_duration(latency: Duration) -> String {
    if latency > Duration::from_secs(1) {
        format!("{:.1}s", latency.as_secs_f64())
    } else if latency > Duration::from_millis(1) {
        format!("{}ms", latency.as_millis())
    } else {
        format!("{}μs", latency.as_micros())
    }
}

pub fn layer_router(router: Router) -> Router {
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
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(CatchPanicLayer::new())
        .layer(CorsLayer::permissive())
        .layer(RequestDecompressionLayer::new())
        .layer(CompressionLayer::new());

    router.layer(layer)
}
