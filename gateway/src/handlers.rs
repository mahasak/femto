use crate::{
    cache::CacheService,
    database::Database,
    errors::AppError,
    models::{
        application::ApplicationResponse,
        health_check::{HealtCheckResponse, HealthCheck},
        merchant_channel::{MerchantChannelEligbleResponse, MerchantChannelResponse},
        search_application::SearchApplication,
    },
    utils::custom_response::{CustomResponseBuilder, CustomResponseResult as Response},
};
use axum::response::IntoResponse;
use axum::{
    body::{Body, Bytes},
    extract::{Json, Query, State},
    http::{header, HeaderName, Request, Response as AxumResponse, StatusCode},
    routing::{get, on, post, MethodFilter},
    Router,
};
use axum_macros::{debug_handler, FromRef};
use emit::{__emit_get_event_data, emit, info};
use hyper::body;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

#[derive(Clone, FromRef)]
pub struct SharedState {
    pub(crate) database: Database,
    pub(crate) cache: CacheService,
}

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
        .route("/", get(root))
        .route("/healthcheck", get(healthcheck_handler))
        .route("/applications", get(get_applications_handler))
        .route("/application", get(get_application_handler))
        .route("/merchants", get(get_merchant_channels_handler))
        .route("/merchant", get(get_merchant_channel_handler))
        .route("/eligible", get(is_merchant_channel_eligible_handler))
        .route("/sequence", get(sequence_handler))
        .route(
            "/webhook/messenger",
            // on(MethodFilter::GET | MethodFilter::POST, messenger_webhook_handler)messenger_verify_subscription_handler
            post(messenger_webhook_handler).get(messenger_verify_subscription_handler),
        )
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().include_headers(true))
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    println!("request: {}", _request.uri());
                    println!("method: {}", _request.method());
                    println!("method: {:#?}", _request.headers());
                    println!("body: {:?}", _request.body().clone());
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

#[debug_handler]
pub async fn root(req: Request<Body>) -> &'static str {
    println!("root request");
    log::info!("request: {:?}", req.headers());
    "Femto Server"
}

#[debug_handler]
pub async fn healthcheck_handler(State(state): State<SharedState>) -> Response<HealtCheckResponse> {
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
pub async fn get_applications_handler(
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
pub async fn get_application_handler(
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

#[debug_handler]
pub async fn is_merchant_channel_eligible_handler(
    State(state): State<SharedState>,
    Query(search): Query<SearchApplication>,
) -> Response<MerchantChannelEligbleResponse> {
    let result = if let Some(id) = &search.id {
        state
            .database
            .is_merchant_channel_eligible(id.to_string())
            .await?
    } else {
        false
    };

    let id = match search.id {
        Some(search) => search,
        None => "n/a".to_string(),
    };

    let res = MerchantChannelEligbleResponse {
        ref_id: id,
        eligible: result,
    };

    let res = CustomResponseBuilder::new()
        .body(res)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

#[debug_handler]
pub async fn get_merchant_channels_handler(
    State(state): State<SharedState>,
) -> Response<Vec<MerchantChannelResponse>> {
    let channels = state.database.get_merchant_channels().await?;
    let channels = channels
        .into_iter()
        .map(Into::into)
        .collect::<Vec<MerchantChannelResponse>>();

    let res = CustomResponseBuilder::new()
        .body(channels)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

#[debug_handler]
pub async fn get_merchant_channel_handler(
    State(state): State<SharedState>,
    Query(search): Query<SearchApplication>,
) -> Response<MerchantChannelResponse> {
    println!("{:?}", search);
    let channel = if let Some(id) = &search.id {
        state.database.get_merchant_channel(id.to_string()).await?
    } else {
        None
    };

    let channel = match channel {
        Some(channel) => MerchantChannelResponse::from(channel),
        None => {
            info!("Application not found, returning 404 status code",);
            return Err(AppError::not_found());
        }
    };

    let res = CustomResponseBuilder::new()
        .body(channel)
        .status_code(StatusCode::OK)
        .build();

    Ok(res)
}

#[debug_handler]
pub async fn sequence_handler(
    State(state): State<SharedState>,
    Query(search): Query<SearchApplication>,
) -> Response<String> {
    let result = if let Some(id) = &search.id {
        state.database.get_sequence(&id.to_string()).await?
    } else {
        0
    };

    let result = CustomResponseBuilder::new()
        .body(result.to_string())
        .status_code(StatusCode::OK)
        .build();

    Ok(result)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessengerVerifysubscriptionParam {
    #[serde(alias = "hub.mode")]
    pub hub_mode: Option<String>,
    #[serde(alias = "hub.verify_token")]
    pub hub_verify_token: Option<String>,
    #[serde(alias = "hub.challenge")]
    pub hub_challenge: Option<String>,
}

#[debug_handler]
pub async fn messenger_verify_subscription_handler(
    Query(query): Query<MessengerVerifysubscriptionParam>,
) -> String {
    let verify_token = match query.hub_verify_token {
        Some(token) => token,
        None => {
            return "No verify token".to_string();
        }
    };

    let hub_mode = match query.hub_mode {
        Some(mode) => mode,
        None => {
            return "No hub mode".to_string();
        }
    };

    let hub_challenge = match query.hub_challenge {
        Some(challenge) => challenge,
        None => {
            return "No hub challenge".to_string();
        }
    };

    if hub_mode == "subscribe" && verify_token == "ITISAGOODDAYTODIE" {
        (hub_challenge.to_string())
    } else {
        ("Veirification failed".to_string())
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Sender {
    id: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct QuickReplyPayload {
    payload: String,
}

impl QuickReplyPayload {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Message {
    text: Option<String>,
    quick_reply: Option<QuickReplyPayload>,
}

impl Message {
    pub fn get_text(&self) -> String {
        self.text.clone().unwrap_or_default()
    }

    pub fn get_quick_reply(&self) -> Option<QuickReplyPayload> {
        self.quick_reply.clone()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Postback {
    payload: String,
}

impl MessagePostback {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct Messaging {
    sender: Sender,
    postback: Option<MessagePostback>,
    message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    messaging: Vec<Messaging>,
}

#[derive(Debug, Deserialize)]
pub struct InComingData {
    object: String,
    entry: Vec<Entry>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePostback {
    payload: String,
}

impl Postback {
    pub fn get_payload(&self) -> &String {
        &self.payload
    }
}

#[debug_handler]
pub async fn messenger_webhook_handler(_payload: Option<Json<InComingData>>) -> String {
    println!("receive message");
    // let (req_parts, req_body) = req.into_parts();
    // let body = req.body();
    // // dbg!(req_parts);
    // // dbg!(req_body);
    // let bytes = buffer_and_print("request", path, body, true).await?;
    // dbg!(body);
    let version = env!("CARGO_PKG_VERSION");
    dbg!(Some(_payload));
    let response = json!({
        "data": {
            "version": version,
        },
        "message": "Service is running..."
    });
    (version.to_string())
}
