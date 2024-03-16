use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::http::StatusCode;
use mongodb::{bson::doc, options::FindOptions, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend_core::features::device_status_feature::models::Device, json::Json};

use super::MONGOC;

#[derive(Serialize, JsonSchema)]
pub struct Response {
    devices: Option<Vec<ResponseDevice>>,
    message: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ResponseDevice {
    id: u32,
    components: Vec<u32>,
}

async fn handler() -> impl IntoApiResponse {
    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    if let Ok(mut device_cursor) = device_coll.find(doc! { }, FindOptions::builder().projection( doc! { "id": 1, "components.id": 1 }).build()).await {
        let mut devices = vec![];
        while let Ok(true) = device_cursor.advance().await {
            let device = device_cursor.deserialize_current();
            match device {
                Ok(device) => devices.push(ResponseDevice { id: device.id, components: device.components.iter().map(|c| c.id).collect() }),
                Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response{ devices: None, message: String::from("Failed to fetch all devices") })),
            }
        }
        (StatusCode::OK, Json(Response{ devices: Some(devices), message: String::from("Successfully fetch all devices") }))
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(Response{ devices: None, message: String::from("Failed to fetch all devices") }))
    }

}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(handler, |op| {
            op.description("Get all devices")
                .tag("Device status")
                .response::<200, Json<Response>>()
                .response::<500, Json<Response>>()
        }),
    )
}

