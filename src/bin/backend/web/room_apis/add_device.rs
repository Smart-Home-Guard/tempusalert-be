use std::collections::HashSet;

use aide::axum::{routing::post_with, ApiRouter};
use axum::http::{HeaderMap, StatusCode};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{backend_core::models::Room, json::Json};

use crate::database_client::{init_database, MONGOC};

use super::utils::{check_device_exist, DeviceCheckExistResult};

#[derive(Deserialize, JsonSchema)]
pub struct AddDeviceBody {
    email: String,
    room_name: String,
    device_ids: Vec<u32>,
}

#[derive(Serialize, JsonSchema)]
pub struct AddDeviceResponse {
    message: String,
}

async fn handler(
    headers: HeaderMap,
    Json(AddDeviceBody {
        email,
        room_name,
        device_ids,
    }): Json<AddDeviceBody>,
) -> (StatusCode, Json<AddDeviceResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(AddDeviceResponse {
                message: String::from("Forbidden"),
            }),
        );
    }

    let mongoc = MONGOC.get_or_init(init_database).await;

    let room_coll: Collection<Room> = mongoc.default_database().unwrap().collection("rooms");

    if let Ok(room_opt) = room_coll
        .find_one(
            doc! { "name": room_name.clone(), "owner_name": email.clone() },
            None,
        )
        .await
    {
        if let Some(mut room) = room_opt {
            if let Some(DeviceCheckExistResult {
                existing_ids,
                non_existing_ids,
            }) = check_device_exist(device_ids.clone(), &email).await
            {
                room.devices.extend(existing_ids.clone());
                let unique_devices: HashSet<u32> = room.devices.iter().cloned().collect();
                room.devices = unique_devices.into_iter().collect();

                if let Err(_) = room_coll
                    .replace_one(
                        doc! { "name": room_name.clone(), "owner_name": email.clone() },
                        room,
                        None,
                    )
                    .await
                {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AddDeviceResponse {
                            message: String::from("Failed to update room"),
                        }),
                    );
                }

                if non_existing_ids.len() == 0 {
                    return (
                        StatusCode::OK,
                        Json(AddDeviceResponse {
                            message: format!(
                                "Added device {:?} to room {}",
                                existing_ids, room_name
                            ),
                        }),
                    );
                }
                return (
                    StatusCode::NOT_FOUND,
                    Json(AddDeviceResponse {
                        message: String::from(format!(
                            "Cannot find device {:?} of user {}",
                            non_existing_ids, email,
                        )),
                    }),
                );
            }
        } else {
            let new_room = Room {
                name: room_name.clone(),
                owner_name: email.clone(),
                devices: device_ids.clone(),
            };

            if let Err(_) = room_coll.insert_one(new_room, None).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AddDeviceResponse {
                        message: String::from("Failed to create room"),
                    }),
                );
            }

            return (
                StatusCode::OK,
                Json(AddDeviceResponse {
                    message: format!(
                        "Created room {} for user {} and added device {:?}",
                        room_name, email, device_ids
                    ),
                }),
            );
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(AddDeviceResponse {
            message: String::from("Internal server error"),
        }),
    )
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(handler, |op| {
            op.description("Add device to room for specific user")
                .tag("Room")
                .response::<200, Json<AddDeviceResponse>>()
                .response::<403, Json<AddDeviceResponse>>()
                .response::<404, Json<AddDeviceResponse>>()
                .response::<500, Json<AddDeviceResponse>>()
        }),
    )
}
