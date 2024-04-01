use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend_core::features::devices_status_feature::models::Device, json::Json};

use super::MONGOC;

#[derive(Serialize, JsonSchema)]
pub struct GetDeviceByIdResponse {
    device: Option<Device>,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetDeviceByIdQuery {
    device_id: u32,
    email: String,
}

async fn handler(
    headers: HeaderMap,
    Query(GetDeviceByIdQuery { device_id, email }): Query<GetDeviceByIdQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetDeviceByIdResponse {
                message: String::from("Forbidden"),
                device: None,
            }),
        );
    }
    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    match device_coll
        .find_one(doc! { "id": device_id, "owner_name": email.clone() }, None)
        .await
    {
        Ok(Some(device)) => (
            StatusCode::OK,
            Json(GetDeviceByIdResponse {
                device: Some(device),
                message: format!("Successfully fetch device '{device_id}'"),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetDeviceByIdResponse {
                device: None,
                message: format!(
                    "No device with id '{}' for user '{}'",
                    device_id,
                    email.clone()
                ),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetDeviceByIdResponse {
                device: None,
                message: format!("Unexpected error while fetching device with id '{device_id}'"),
            }),
        ),
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/devices/:device_id",
        get_with(handler, |op| {
            op.description("Get devices by id for a given user by email")
                .tag("Devices status")
                .response::<200, Json<GetDeviceByIdResponse>>()
                .response::<403, Json<GetDeviceByIdResponse>>()
                .response::<500, Json<GetDeviceByIdResponse>>()
        }),
    )
}
