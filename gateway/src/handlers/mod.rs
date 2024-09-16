use crate::{cache::CacheService, database::Database, handlers::state::SharedState};
use axum::{
    body::Body,
    http::{header,Method, Request, Response as AxumResponse},
    Router,
};
use emit::{__emit_get_event_data, emit, error, info};
use std::time::Duration;
use slog::Logger;
use tower_request_id::{RequestId, RequestIdLayer};
use tower_http::{
    classify::ServerErrorsFailureClass,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::TraceLayer
};
use tracing::Span;
use context::{RequestContext};
use crate::handlers::context::context_middleware;

pub mod api;
pub mod messenger;
pub mod state;
mod context;

pub fn router(database: Database, cache: CacheService, logger: Logger) -> Router {
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
                .on_request(|request: &Request<Body>, _span: &Span| {
                    let trace_id = request.extensions().get::<RequestId>().unwrap();
                    info!("incoming request, request ID {}, URL: {},", request_id: trace_id.to_string(), url: request.uri().to_string());
                })
                .on_response(
                    |response: &AxumResponse<Body>, latency: Duration, _span: &Span| {
                        let in_ms =
                            latency.as_secs() * 1000 + latency.subsec_nanos() as u64 / 1_000_000;
                        let request_context = response.extensions().get::<RequestContext>().unwrap();
                        info!("request processed in ms {}, request ID {}, URL: {}", response_time: in_ms, request_id: request_context.request_id, request_uri: request_context.uri.to_string());
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        error!("server error, error: {}", error: error.to_string());
                    },
                ),
        )
        .layer(RequestIdLayer)
        .layer(CompressionLayer::new())
        .with_state(SharedState { database, cache, logger })
}

fn get_cors_layer() -> CorsLayer {
    CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
}


