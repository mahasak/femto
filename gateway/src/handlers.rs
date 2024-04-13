use crate::{
    cache::Cache,
    database::Database,
    errors::AppError,
    models::{
        application::ApplicationResponse,
        health_check::{HealtCheckResponse, HealthCheck},
        search_application::SearchApplication,
    },
    utils::custom_response::{CustomResponseBuilder, CustomResponseResult as Response},
};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{header, HeaderName, Request, Response as AxumResponse, StatusCode},
    routing::get,
    Router,
};
use axum_macros::{debug_handler, FromRef};
use emit::{__emit_get_event_data, emit, info};
use std::time::Duration;
use tower_http::request_id::{MakeRequestId, RequestId, SetRequestIdLayer};
use tower_http::{
    classify::ServerErrorsFailureClass,
    cors::{Any, CorsLayer},
};
use tower_http::{
    compression::CompressionLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveHeadersLayer, trace,
};
use tracing::Span;

#[derive(Clone, FromRef)]
pub struct SharedState {
    pub(crate) database: Database,
    pub(crate) cache: Cache,
}

#[derive(Clone, Default)]
struct MyMakeRequestId;

impl MakeRequestId for MyMakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = uuid::Uuid::new_v4().to_string().parse().unwrap();

        Some(RequestId::new(request_id))
    }
}

pub fn router(database: Database, cache: Cache) -> Router {
    let cors = CorsLayer::new().allow_origin(Any);
    let x_request_id = HeaderName::from_static("x-request-id");
    Router::new()
        .route("/", get(root))
        .route("/healthcheck", get(healthcheck))
        .route("/applications", get(get_applications))
        .route("/application", get(get_application))
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    trace::DefaultOnRequest::new().level(tracing::Level::INFO);
                })
                .on_response(
                    |_response: &AxumResponse<Body>, latency: Duration, _span: &Span| {
                        let in_ms =
                            latency.as_secs() * 1000 + latency.subsec_nanos() as u64 / 1_000_000;

                        info!("response in ms: {}", response_time: in_ms);
                    },
                )
                .on_failure(
                    |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
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

#[debug_handler]
pub async fn root(req: Request<Body>) -> &'static str {
    println!("root request");
    log::info!("request: {:?}", req.headers());
    "Femto Server"
}

#[debug_handler]
pub async fn healthcheck(State(state): State<SharedState>) -> Response<HealtCheckResponse> {
    println!("health check request");
    let date_now = state.database.get_now().await?;
    let ping = state.cache.ping().await?;
    let health_check_response = HealthCheck::new(date_now, ping);
    let res = HealtCheckResponse::from(health_check_response);

    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::OK)
        .build();
    Ok(res)
}

#[debug_handler]
pub async fn get_applications(
    State(state): State<SharedState>,
) -> Response<Vec<ApplicationResponse>> {
    let apps = state.database.get_applications().await?;
    let apps = apps
        .into_iter()
        .map(Into::into)
        .collect::<Vec<ApplicationResponse>>();

    let res = CustomResponseBuilder::new()
        .body(apps)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

#[debug_handler]
pub async fn get_application(
    State(state): State<SharedState>,
    Query(search): Query<SearchApplication>,
) -> Response<ApplicationResponse> {
    println!("{:?}", search);
    let app = if let Some(id) = &search.id {
        state.database.get_application(id.to_string()).await?
    } else {
        None
    };

    let app = match app {
        Some(app) => ApplicationResponse::from(app),
        None => {
            info!("Application not found, returning 404 status code",);
            return Err(AppError::not_found());
        }
    };

    let res = CustomResponseBuilder::new()
        .body(app)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}
