use std::sync::Arc;

use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query, http::{HeaderMap, StatusCode},
};
use mongodb::bson::doc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex};

use crate::{backend_core::features::remote_control_feature::{models::*, IotNotification, WebFeature, WebNotification}, json::Json};

#[derive(Deserialize, JsonSchema)]
pub struct ControlLightQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ControlLightResponse {
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ControlLightRequestBody {
    device_id: usize,
    component_id: usize,
    command: LightCommand,
}

static mut IOT_TX: Option<Sender<WebNotification>> = None;
static mut IOT_RX: Option<Arc<Mutex<Receiver<IotNotification>>>> = None;

async fn handler(
    headers: HeaderMap,
    Query(ControlLightQuery { email }): Query<ControlLightQuery>,
    Json(ControlLightRequestBody { device_id, component_id, command }): Json<ControlLightRequestBody>,
) -> impl IntoApiResponse {
    let iot_tx = unsafe {
        IOT_TX.clone()
    };

    let iot_rx = unsafe {
        IOT_RX.clone()
    };

    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str()) {  
        return (
            StatusCode::FORBIDDEN,
            Json(ControlLightResponse {
                message: String::from("Forbidden"),
            }),
        );
    }
        
    let notif = WebNotification::LightCommandNotification { device_id, component_id, command, };
        
    if let Ok(_) = iot_tx.clone().unwrap().send(notif).await {
        if let Some(response) = iot_rx.clone().unwrap().lock().await.recv().await {
            return (
                response.status_code,
                Json(ControlLightResponse {
                    message: response.message,
                }),
            );
        };
    }
         
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ControlLightResponse {
            message: String::from("Internal server error"),
        }),
    );
}

pub fn routes(web_feature_instance: &mut WebFeature) -> ApiRouter {
    unsafe {
        IOT_RX = Some(web_feature_instance.iot_rx.clone());
        IOT_TX = Some(web_feature_instance.iot_tx.clone());
    }

    ApiRouter::new().api_route(
        "/light",
        post_with(handler, |op| {
            op.description("Control a light by email")
                .tag("Remote control")
                .response::<200, Json<ControlLightResponse>>()
                .response::<403, Json<ControlLightResponse>>()
                .response::<500, Json<ControlLightResponse>>()
        }),
    )
}
