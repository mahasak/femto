use std::env;

use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use axum_macros::debug_handler;

use crate::{handlers::state::SharedState, models::messenger_webhook::MessengerWebhook};

use axum::extract::Query;

use crate::models::messenger_webhook::MessengerVerifysubscription;

pub fn create_route() -> Router<SharedState> {
    Router::new()
        .route("/webhook/messenger", post(messenger_post_handler))
        .route("/webhook/messenger", get(messenger_get_handler))
}

#[debug_handler]
async fn messenger_get_handler(
    State(_state): State<SharedState>,
    Query(query): Query<MessengerVerifysubscription>,
) -> String {
    let fb_verify_token = env::var("FACEBOOK_WEBHOOK_VERIFY_TOKEN").expect("env::FACEBOOK_WEBHOOK_VERIFY_TOKEN is missing");
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

    if hub_mode == "subscribe" && verify_token == fb_verify_token {
        hub_challenge.to_string()
    } else {
        "Veirification failed".to_string()
    }
}

#[debug_handler]
async fn messenger_post_handler(
    State(_state): State<SharedState>,
    _payload: Option<Json<MessengerWebhook>>,
) -> String {
    println!("receive message");
    let version = env!("CARGO_PKG_VERSION");
    dbg!(Some(_payload));
    version.to_string()
}
