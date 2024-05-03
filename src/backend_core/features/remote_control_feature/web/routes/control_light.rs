use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query, http::{HeaderMap, StatusCode},
};
use mongodb::bson::doc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{auth::get_client_id_from_web_token, backend_core::features::{remote_control_feature::{models::*, notifications::RemoteControlIotNotification, web::WebRemoteControlFeature, WebNotification}, WebFeature}, json::Json};

use super::WEB_INSTANCE;

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

async fn handler(
    headers: HeaderMap,
    Query(ControlLightQuery { email }): Query<ControlLightQuery>,
    Json(ControlLightRequestBody { device_id, component_id, command }): Json<ControlLightRequestBody>,
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
            Json(ControlLightResponse {
                message: String::from("Forbidden"),
            }),
        );
    }
        
    let jwt = String::from(headers.get("jwt").unwrap().to_str().unwrap());
    let client_id = get_client_id_from_web_token(web_instance.jwt_key.as_str(), jwt, &mut web_instance.mongoc).await.unwrap();

    let notif = WebNotification::LightCommandNotification { device_id, component_id, command, client_id };
        
    if let Ok(response) = serde_json::from_str::<RemoteControlIotNotification>(
        &web_instance.clone().send_message_to_iot(serde_json::to_string(&notif).unwrap()).await
    ) {
        return (
            { if response.status_code == 200 { StatusCode::OK } else { StatusCode::BAD_REQUEST } },
            Json(ControlLightResponse {
                message: response.message,
            }),
        );
    }
         
    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ControlLightResponse {
            message: String::from("Internal server error"),
        }),
    );
}

pub fn routes() -> ApiRouter {
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
