use crate::{cache::CacheService, database::Database, handlers::state::SharedState};
use axum::{
    body::Body,
    http::{header, HeaderName, Request, Response as AxumResponse},
    Router,
};
use emit::{__emit_get_event_data, emit, info};
use std::time::Duration;
use tower_http::{
    classify::ServerErrorsFailureClass,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    propagate_header::PropagateHeaderLayer,
    request_id::{MakeRequestId, RequestId, SetRequestIdLayer},
    sensitive_headers::SetSensitiveHeadersLayer,
    trace,
};
use tracing::Span;

pub mod api;
pub mod messenger;
pub mod state;

#[derive(Clone, Default)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = uuid::Uuid::new_v4().to_string().parse().unwrap();

        Some(RequestId::new(request_id))
    }
}

pub fn router(database: Database, cache: CacheService) -> Router {
    let cors = CorsLayer::new().allow_origin(Any);
    let x_request_id = HeaderName::from_static("x-request-id");
    Router::new()
        .merge(self::api::create_route())
        .merge(self::messenger::create_route())
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    println!("request id : ,request: {}", _request.uri());
                    println!("method: {}", _request.method());
                    println!("headers: {:#?}", _request.headers());
                    info!("requested URL: {}", url: _request.uri().to_string());
                    trace::DefaultOnRequest::new().level(tracing::Level::INFO);
                })
                .on_response(
                    |_response: &AxumResponse<Body>, latency: Duration, _span: &Span| {
                        let in_ms =
                            latency.as_secs() * 1000 + latency.subsec_nanos() as u64 / 1_000_000;
                        println!("headers: {:#?}", _response.headers());
                        info!("response in ms: {}", response_time: in_ms);
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        println!("error");
                        tracing::error!("error: {}", error);
                    },
                ),
        )
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MyMakeRequestId::default(),
        ))
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))
        .layer(CompressionLayer::new())
        .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
            "x-request-id",
        )))
        .layer(cors)
        .with_state(SharedState { database, cache })
}
