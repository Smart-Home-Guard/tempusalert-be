use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, options::FindOptions, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::MONGOC;
use crate::{
    backend_core::features::devices_status_feature::{
        iot::mqtt_messages::ComponentType, models::Device,
    },
    json::Json,
};

#[derive(Deserialize, JsonSchema)]
pub struct GetAllDevicesQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetAllDeviceResponse {
    devices: Option<Vec<ResponseDevice>>,
    message: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ResponseComponent {
    component: u32,
    kind: ComponentType,
}

#[derive(Serialize, JsonSchema)]
pub struct ResponseDevice {
    id: u32,
    components: Vec<ResponseComponent>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetAllDevicesQuery { email }): Query<GetAllDevicesQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetAllDeviceResponse {
                message: String::from("Forbidden"),
                devices: None,
            }),
        );
    }
    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    if let Ok(mut device_cursor) = device_coll
        .find(
            doc! { "owner_name": email.clone() },
            None,
        )
        .await
    {
        let mut devices = vec![];
        while let Ok(true) = device_cursor.advance().await {
            let device = device_cursor.deserialize_current();
            match device {
                Ok(device) => devices.push(ResponseDevice {
                    id: device.id,
                    components: device
                        .components
                        .iter()
                        .map(|c| ResponseComponent {
                            component: c.id,
                            kind: c.kind.clone(),
                        })
                        .collect(),
                }),
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(GetAllDeviceResponse {
                            devices: None,
                            message: format!(
                                "Failed to fetch all devices for user '{}'",
                                email.clone()
                            ),
                        }),
                    )
                }
            }
        }
        (
            StatusCode::OK,
            Json(GetAllDeviceResponse {
                devices: Some(devices),
                message: format!(
                    "Successfully fetch all devices for user '{}'",
                    email.clone()
                ),
            }),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetAllDeviceResponse {
                devices: None,
                message: format!("Failed to fetch all devices for user '{}'", email.clone()),
            }),
        )
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/devices",
        get_with(handler, |op| {
            op.description("Get all devices for a given user by email")
                .tag("Devices status")
                .response::<200, Json<GetAllDeviceResponse>>()
                .response::<403, Json<GetAllDeviceResponse>>()
                .response::<500, Json<GetAllDeviceResponse>>()
        }),
    )
}
