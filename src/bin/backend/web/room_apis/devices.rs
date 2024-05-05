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
pub struct DeviceIdentifiersBody {
    email: String,
    room_name: String,
    device_ids: Vec<u32>,
}

#[derive(Serialize, JsonSchema)]
pub struct NotificationMessageResponse {
    message: String,
}

async fn add_device_handler(
    headers: HeaderMap,
    Json(DeviceIdentifiersBody {
        email,
        room_name,
        device_ids,
    }): Json<DeviceIdentifiersBody>,
) -> (StatusCode, Json<NotificationMessageResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(NotificationMessageResponse {
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
                        Json(NotificationMessageResponse {
                            message: String::from("Failed to update room"),
                        }),
                    );
                }

                if non_existing_ids.len() == 0 {
                    return (
                        StatusCode::OK,
                        Json(NotificationMessageResponse {
                            message: format!(
                                "Added device {:?} to room {}",
                                existing_ids, room_name
                            ),
                        }),
                    );
                }
                return (
                    StatusCode::NOT_FOUND,
                    Json(NotificationMessageResponse {
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
                    Json(NotificationMessageResponse {
                        message: String::from("Failed to create room"),
                    }),
                );
            }

            return (
                StatusCode::OK,
                Json(NotificationMessageResponse {
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
        Json(NotificationMessageResponse {
            message: String::from("Internal server error"),
        }),
    )
}

async fn remove_device_from_room_handler(
    headers: HeaderMap,
    Json(DeviceIdentifiersBody {
        email,
        room_name,
        device_ids,
    }): Json<DeviceIdentifiersBody>,
) -> (StatusCode, Json<NotificationMessageResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(NotificationMessageResponse {
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
            let mut existing_ids = Vec::new();
            let mut non_existing_ids = device_ids.clone();

            for (_index, device_id) in room.devices.iter().enumerate() {
                if let Some(pos) = non_existing_ids.iter().position(|&id| id == *device_id) {
                    existing_ids.push(*device_id);
                    non_existing_ids.remove(pos);
                }
            }

            room.devices.retain(|&id| !existing_ids.contains(&id));

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
                    Json(NotificationMessageResponse {
                        message: String::from("Failed to update room"),
                    }),
                );
            }

            if non_existing_ids.len() == 0 {
                return (
                    StatusCode::OK,
                    Json(NotificationMessageResponse {
                        message: format!(
                            "Successfully removed devices {:?} from room {}",
                            existing_ids, room_name
                        ),
                    }),
                );
            }
            return (
                StatusCode::NOT_FOUND,
                Json(NotificationMessageResponse {
                    message: String::from(format!(
                        "Cannot find device {:?} of user {}",
                        non_existing_ids, email,
                    )),
                }),
            );
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(NotificationMessageResponse {
                    message: format!("Cannot find room {} of user {}", room_name, email),
                }),
            );
        }
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(NotificationMessageResponse {
            message: String::from("Internal server error"),
        }),
    )
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(add_device_handler, |op| {
            op.description("Add devices to room for specific user")
                .tag("Room")
                .response::<200, Json<NotificationMessageResponse>>()
                .response::<403, Json<NotificationMessageResponse>>()
                .response::<404, Json<NotificationMessageResponse>>()
                .response::<500, Json<NotificationMessageResponse>>()
        })
        .delete_with(remove_device_from_room_handler, |op| {
            op.description("Remove devices from room for specific user")
                .tag("Room")
                .response::<200, Json<NotificationMessageResponse>>()
                .response::<403, Json<NotificationMessageResponse>>()
                .response::<404, Json<NotificationMessageResponse>>()
                .response::<500, Json<NotificationMessageResponse>>()
        }),
    )
}
