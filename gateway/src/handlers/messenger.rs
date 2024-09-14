use std::env;
use axum::{
    extract::Query,
    routing::{get, post},
    Router, Json,
    };
use axum::extract::State;
use axum_macros::debug_handler;


use emit::{__emit_get_event_data, emit, info};
use crate::{handlers::state::SharedState, models::messenger_webhook::MessengerWebhook};
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
    State(state): State<SharedState>,
    Json(payload): Json<MessengerWebhook>,

) -> String {

    let work_payload = payload.clone();
    let object = work_payload.object;
    //let page_id = Some(payload.   entry);

    if object == "page" {

        for entry in work_payload.entry.iter() {
            let page_id = entry.id.clone();
            let eligible = state
                .database
                .is_merchant_channel_eligible(page_id.clone())
                .await.unwrap();

            if eligible {
                info!("Eligibility: {}, Page ID {} is eligible", eligible: eligible, page_id: page_id.clone());

                let app_config = state.database.get_merchant_config(page_id.clone()).await.unwrap();
                match  app_config {
                    Some(app_config) => {
                        info!("Page {} configuration, topic: {}, app_id: {}, enabled: {}",
                            page_id: page_id,
                            topic: app_config.topic,
                            app_id: app_config.app_id,
                            enabled: app_config.enabled);
                        let json_payload = payload.clone();
                        let json_str = serde_json::to_string(&json_payload).unwrap();
                        info!("receiving message: {}", webhook_payload: json_str);
                        let _ = state.cache.publish(app_config.topic, json_str).await.unwrap();
                    }
                    None => {
                        info!("No merchant config for page ID {} not found", page_id: page_id);
                    }
                }
            } else {
                info!("Eligibility: {}, Page ID {} is NOT eligible", eligible: eligible, page_id: page_id.clone());
            }
        }
    } else {
        info!("Received non-page object, Got {}", object: object);
    }

    // println!("{:?}", serde_json::to_string(&debug_obj).unwrap());



    "{\"success\":true}".to_string()
}
