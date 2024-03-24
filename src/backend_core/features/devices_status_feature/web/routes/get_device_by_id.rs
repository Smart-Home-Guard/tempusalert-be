use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformParameter,
};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{backend_core::features::devices_status_feature::models::Device, json::Json};

use super::MONGOC;

#[derive(Serialize, JsonSchema)]
pub struct Response {
    device: Option<Device>,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct Params {
    device_id: u32,
    email: String,
}

async fn handler(
    headers: HeaderMap,
    Path(Params { device_id, email }): Path<Params>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(Response {
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
                    email.clone()
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
        "/:email/devices/:device_id",
        get_with(handler, |op| {
            op.description("Get devices by id for a given user by email")
                .tag("Devices status")
                .parameter("email", |op: TransformParameter<String>| {
                    op.description("The registered email")
                })
                .parameter("device_id", |op: TransformParameter<u32>| {
                    op.description("The id of a device owned by the user")
                })
                .response::<200, Json<Response>>()
                .response::<403, Json<Response>>()
                .response::<500, Json<Response>>()
        }),
    )
}
