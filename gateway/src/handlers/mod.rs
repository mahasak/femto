use crate::{cache::CacheService, database::Database, handlers::state::SharedState};
use axum::{
    body::Body,
    http::{header, Request, Response as AxumResponse},
    Router,
};
use emit::{__emit_get_event_data, emit, info};
use std::time::Duration;
use axum::http::{Method};
use tower_request_id::{RequestId, RequestIdLayer};
use tower_http::{classify::ServerErrorsFailureClass, compression::CompressionLayer, cors::{Any, CorsLayer}, sensitive_headers::SetSensitiveHeadersLayer, trace};
use tower_http::trace::{TraceLayer};
use tracing::{Level, Span};
use context::{RequestContext};
use crate::handlers::context::context_middleware;

pub mod api;
pub mod messenger;
pub mod state;
mod context;

pub fn router(database: Database, cache: CacheService) -> Router {
    Router::new()
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))

        .layer(get_cors_layer())
        .merge(self::api::create_route())
        .merge(self::messenger::create_route())
        .layer(axum::middleware::from_fn(context_middleware))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    let request_id = uuid::Uuid::new_v4();
                    println!("request id: {}", request_id);
                    tracing::span!(
                          Level::DEBUG,
                          "request",
                          method = tracing::field::display(request.method()),
                          uri = tracing::field::display(request.uri()),
                          version = tracing::field::debug(request.version()),
                          request_id = tracing::field::display(request_id)
                      )
                })
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    let trace_id = _request.extensions().get::<RequestId>().unwrap();
                    info!("requested URL: {}", url: _request.uri().to_string());
                    info!("requested ID: {}", request_id: trace_id.to_string());

                    trace::DefaultOnRequest::new().level(tracing::Level::INFO);
                })
                .on_response(
                    |_response: &AxumResponse<Body>, latency: Duration, _span: &Span| {
                        let in_ms =
                            latency.as_secs() * 1000 + latency.subsec_nanos() as u64 / 1_000_000;
                        let request_context = _response.extensions().get::<RequestContext>().unwrap();
                        info!("response in ms: {}", response_time: in_ms);
                        info!("request URI : {} ", request_uri: request_context.uri.to_string());
                        info!("request ID : {} ", request_id: request_context.request_id);
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        println!("error");
                        tracing::error!("error: {}", error);
                    },
                ),
        )
        .layer(RequestIdLayer)
        .layer(CompressionLayer::new())
        .with_state(SharedState { database, cache })
}

fn get_cors_layer() -> CorsLayer {
    CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
}


