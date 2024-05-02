use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, options::FindOptions, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend_core::features::remote_control_feature::models::*, json::Json};

use super::MONGOC;

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
pub struct ResponseDevice {
    id: u32,
    components: Vec<u32>,
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
            FindOptions::builder()
                .projection(doc! { "id": 1, "components.id": 1 })
                .build(),
        )
        .await
    {
        let mut devices = vec![];
        while let Ok(true) = device_cursor.advance().await {
            let device = device_cursor.deserialize_current();
            match device {
                Ok(device) => devices.push(ResponseDevice {
                    id: device.id,
                    components: device.components.iter().map(|c| c.id).collect(),
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
