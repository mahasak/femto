use std::env;
use axum::{
    async_trait,
    extract::{FromRef, FromRequest, FromRequestParts, Query},
    http::{request::Parts, Request},
    routing::{get, post},
    Extension, Router, RequestExt,Json,
    response::{IntoResponse, Response}
    };
use axum::extract::State;
use axum_macros::debug_handler;
use sqlx::{postgres::PgPoolOptions, query, PgPool};
use crate::{handlers::state::SharedState, models::messenger_webhook::MessengerWebhook};

use serde_json::json;
use tracing::field::debug;
use emit::{__emit_get_event_data, emit, error, info, debug};
use tower_request_id::RequestId;
use crate::handlers::context::RequestContext;
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
struct MyExtractor<T> {
    request_id: RequestId,
    payload: T,
}
#[debug_handler]
async fn messenger_post_handler(
    State(state): State<SharedState>,
    Json(payload): Json<MessengerWebhook>,

) -> String {
    let debug_obj = payload.clone();
    let json = serde_json::to_string(&debug_obj).unwrap();
    info!("receiving message: {}", webhook_payload: json);

    let object = payload.object;
    //let page_id = Some(payload.   entry);

    if object == "page" {
        let page_id = payload.entry[0].id.clone();

        let eligible = state
            .database
            .is_merchant_channel_eligible(page_id.clone())
            .await.unwrap();

        if eligible {
            info!("Page ID {} is eligible", page_id: page_id.clone());

            let app_config = state.database.get_merchant_config(page_id.clone()).await.unwrap();
            match  app_config {
                Some(app_config) => {
                    println!("{:#?}", app_config);
                }
                None => {}
            }

//             let query = query!(r#"select a.id as channel_id, b.app_id as "app_id!", c.topic, c.enabled, a.token
// from merchant_channel a
// inner join application_registry b on a.id = b.channel_id
// inner join public.application c on b.app_id= c.id where ref_id = $1
// "#, page_id.clone()).fetch_one(&state.database.client).await.unwrap();
//             println!("{:#?}", query);

            let app = if let Some(id) = Some(page_id.clone()) {
                println!("{:#}", id.to_string());
                state.database.get_application(id.to_string()).await.unwrap()
            } else {
                None
            };

            match app {
                Some(app) => {
                    info!("Handler for page_id {} is topic {}", page_id: page_id.clone(), topic: app.topic);

                },
                None => {
                    info!("Application not found, returning 404 status code",);
                }
            };


        }
    }

    
    // match  payload.entry {
    //   Some(entry) => println!("{:?}", entry),
    //   None => println!("Not match")
    // }

    println!("{:?}", serde_json::to_string(&debug_obj).unwrap());

    println!("{:?}", object);

    payload.entry.iter().for_each(|entry| {
      println!("{:?}", entry);
    });

    "{\"success\":true}".to_string()
}
