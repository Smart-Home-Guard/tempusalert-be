use std::sync::Arc;

use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query, http::{HeaderMap, StatusCode},
};
use mongodb::bson::doc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex};

use crate::{auth::get_client_id_from_web_token, backend_core::features::remote_control_feature::{models::*, IotNotification, WebFeature, WebNotification}, json::Json};

use super::WEB_INSTANCE;

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

async fn handler(
    headers: HeaderMap,
    Query(ControlBuzzerQuery { email }): Query<ControlBuzzerQuery>,
    Json(ControlBuzzerRequestBody { device_id, component_id, command }): Json<ControlBuzzerRequestBody>,
) -> impl IntoApiResponse {
    let mut web_instance = unsafe {
        WEB_INSTANCE.clone().unwrap()        
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
        
    let jwt = String::from(headers.get("jwt").unwrap().to_str().unwrap());
    let client_id = unsafe {
        get_client_id_from_web_token(web_instance.jwt_key.as_str(), jwt, &mut web_instance.mongoc).await.unwrap()
    };

    let notif = WebNotification::BuzzerCommandNotification { device_id, component_id, command, client_id };
    
    if let Ok(_) = .await {
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
