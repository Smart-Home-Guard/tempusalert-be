use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{extract::Path, http::StatusCode};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend_core::features::device_status_feature::models::Device, json::Json};

use super::MONGOC;

#[derive(Serialize, JsonSchema)]
pub struct Response {
    device: Option<Device>,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    device_id: u32,
    username: String,
}

async fn handler(
    Path(Params {
        device_id,
        username,
    }): Path<Params>,
) -> impl IntoApiResponse {
    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    match device_coll
        .find_one(
            doc! { "id": device_id, "owner_name": username.clone() },
            None,
        )
        .await
    {
        Ok(Some(device)) => (
            StatusCode::OK,
            Json(Response {
                device: Some(device),
                message: format!("Successfully fetch device '{device_id}'"),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(Response {
                device: None,
                message: format!(
                    "No device with id '{}' for user '{}'",
                    device_id,
                    username.clone()
                ),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response {
                device: None,
                message: format!("Unexpected error while fetching device with id '{device_id}'"),
            }),
        ),
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/:username/devices/:device_id",
        get_with(handler, |op| {
            op.description("Get devices by id for a given username")
                .tag("Device status")
                .response::<200, Json<Response>>()
                .response::<500, Json<Response>>()
        }),
    )
}
