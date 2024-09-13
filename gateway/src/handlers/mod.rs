use std::sync::Arc;
use crate::{cache::CacheService, database::Database, handlers::state::SharedState};
use axum::{
    body::Body,body::Bytes,
    http::{header, HeaderName, Request, Response as AxumResponse},
    Router,
};
use emit::{__emit_get_event_data, emit, info};
use std::time::Duration;
use axum::http::{HeaderValue, Method, Response, Uri};
use axum::middleware::Next;
use tower_request_id::{RequestId, RequestIdLayer};
use tower::ServiceBuilder;
use tower::timeout::TimeoutLayer;
use tower_http::{classify::ServerErrorsFailureClass, compression::CompressionLayer, cors::{Any, CorsLayer}, propagate_header::PropagateHeaderLayer, request_id::{MakeRequestId, SetRequestIdLayer}, sensitive_headers::SetSensitiveHeadersLayer, trace, LatencyUnit};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, OnFailure, OnRequest, OnResponse, TraceLayer};
use tracing::{Level, Span};

pub mod api;
pub mod messenger;
pub mod state;

#[derive(Clone, Default)]
struct MyMakeRequestId;

// impl MakeRequestId for MyMakeRequestId {
//     fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
//         Some(RequestId::new(uuid::Uuid::new_v4().to_string().parse().unwrap()))
//     }
// }

pub fn router(database: Database, cache: CacheService) -> Router {

    let x_request_id = HeaderName::from_static("x-request-id");

    // let middleware = ServiceBuilder::new()
    //     .layer(SetRequestIdLayer::new(
    //         x_request_id.clone(),
    //         MyMakeRequestId::default(),
    //     ))
    //     .layer(SetSensitiveHeadersLayer::new(vec![
    //         header::AUTHORIZATION,
    //         header::COOKIE,
    //     ]))
    //     // Add high level tracing/logging to all requests
    //     .layer(
    //         TraceLayer::new_for_http()
    //             .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
    //                 tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
    //             })
    //             .make_span_with(DefaultMakeSpan::new().include_headers(true))
    //             .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
    //     )
    //     .layer(PropagateHeaderLayer::new(header::HeaderName::from_static(
    //         "x-request-id",
    //     )))
    //     // Set a timeout
    //     .layer(TimeoutLayer::new(Duration::from_secs(10)))
    //     // Compress responses
    //     .layer(CompressionLayer::new());

    Router::new()
        .layer(SetSensitiveHeadersLayer::new(std::iter::once(
            header::AUTHORIZATION,
        )))

        .layer(get_cors_layer())
        .merge(self::api::create_route())
        .merge(self::messenger::create_route())
        .layer(axum::middleware::from_fn(uri_middleware))
        //.layer(axum::middleware::from_fn(request_id_middleware))
        .layer(
            trace::TraceLayer::new_for_http()
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
                    // println!("request id : ,request: {}", _request.uri());
                    // println!("method: {}", _request.method());
                    // println!("method: {:#?}", _request);
                    // println!("headers: {:#?}", _request.headers());
                    // println!("headers: {:#?}", _span);
                    let trace_id = _request.extensions().get::<RequestId>().unwrap();
                    info!("requested URL: {}", url: _request.uri().to_string());
                    info!("requested ID: {}", request_id: trace_id.to_string());

                    trace::DefaultOnRequest::new().level(tracing::Level::INFO);
                })
                .on_response(
                    |_response: &AxumResponse<Body>, latency: Duration, _span: &Span| {
                        let in_ms =
                            latency.as_secs() * 1000 + latency.subsec_nanos() as u64 / 1_000_000;
                        //let trace_id = _response.extensions().get::<RequestId>().unwrap();
                        // println!("headers: {:#?}", _response.headers());
                        let request = _response.extensions().get::<RequestUri>().map(|r| &r.0).unwrap();
                        let request_id = _response.extensions().get::<RequestIdInReponse>().map(|r| &r.0).unwrap();
                        info!("response in ms: {}", response_time: in_ms);
                        info!("request URI : {} ", request_uri: request.to_string());
                        info!("request ID : {} ", request_id: request_id.to_string());
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


#[derive(Clone)]
struct RequestUri(Uri);
#[derive(Clone)]
struct RequestIdInReponse(String);
async fn uri_middleware(request: axum::http::Request<Body>, next: Next) -> axum::response::Response {
    let uri = request.uri().clone();
    let request_ext = request.extensions().clone();
    let request_id = request_ext.get::<RequestId>().map(|r| &r.0).unwrap();

    let mut response = next.run(request).await;

    response.extensions_mut().insert(RequestUri(uri));

    response.extensions_mut().insert(RequestIdInReponse(request_id.to_string()));

    response
}


async fn request_id_middleware(request: axum::http::Request<Body>, next: Next) -> axum::response::Response {
    let uri = request.uri().clone();

    let mut response = next.run(request).await;

    response.extensions_mut().insert(RequestId);

    response
}