use crate::{
    cache::Cache,
    database::Database,
    models::health_check::{HealtCheckResponse, HealthCheck},
    utils::custom_response::{CustomResponseBuilder, CustomResponseResult as Response},
};
use axum::{extract::State, http::StatusCode};
use axum_macros::{debug_handler, FromRef};

#[derive(Clone, FromRef)]
pub struct SharedState {
    pub(crate) database: Database,
    pub(crate) cache: Cache,
}

#[debug_handler]
pub async fn root() -> &'static str {
    "Femto Server"
}

#[debug_handler]
pub async fn healthcheck(State(state): State<SharedState>) -> Response<HealtCheckResponse> {
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
