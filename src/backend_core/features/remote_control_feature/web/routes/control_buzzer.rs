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
pub struct ControlBuzzerQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ControlBuzzerResponse {
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct ControlBuzzerRequestBody {
    device_id: usize,
    component_id: usize,
    command: BuzzerCommand,
}

static mut IOT_TX: Option<Sender<WebNotification>> = None;
static mut IOT_RX: Option<Arc<Mutex<Receiver<IotNotification>>>> = None;

async fn handler(
    headers: HeaderMap,
    Query(ControlBuzzerQuery { email }): Query<ControlBuzzerQuery>,
    Json(ControlBuzzerRequestBody { device_id, component_id, command }): Json<ControlBuzzerRequestBody>,
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
            Json(ControlBuzzerResponse {
                message: String::from("Forbidden"),
            }),
        );
    }
        
    let notif = WebNotification::BuzzerCommandNotification { device_id, component_id, command, };
    
    if let Ok(_) = iot_tx.clone().unwrap().send(notif).await {
        if let Some(response) = iot_rx.clone().unwrap().lock().await.recv().await {
            return (
                response.status_code,
                Json(ControlBuzzerResponse {
                    message: response.message,
                }),
            );
        };
    }
         
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ControlBuzzerResponse {
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
        "/buzzer",
        post_with(handler, |op| {
            op.description("Control a specific buzzer by email")
                .tag("Remote control")
                .response::<200, Json<ControlBuzzerResponse>>()
                .response::<403, Json<ControlBuzzerResponse>>()
                .response::<500, Json<ControlBuzzerResponse>>()
        }),
    )
}
