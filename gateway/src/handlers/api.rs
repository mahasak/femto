use crate::{
    errors::AppError,
    handlers::state::SharedState,
    models::{
        application::ApplicationResponse,
        health_check::{HealtCheckResponse, HealthCheck},
        merchant_channel::{MerchantChannelEligbleResponse, MerchantChannelResponse},
        search_application::SearchApplication,
    },
    utils::custom_response::{CustomResponseBuilder, CustomResponseResult as Response},
};
use axum::{
    body::Body,
    extract::{Query, State},
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use emit::{__emit_get_event_data, emit, info};

pub fn create_route() -> Router<SharedState> {
    Router::new()
        .route("/", get(root))
        .route("/healthcheck", get(healthcheck_handler))
        .route("/applications", get(get_applications_handler))
        .route("/application", get(get_application_handler))
        .route("/merchants", get(get_merchant_channels_handler))
        .route("/merchant", get(get_merchant_channel_handler))
        .route("/eligible", get(is_merchant_channel_eligible_handler))
        .route("/sequence", get(sequence_handler))
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
