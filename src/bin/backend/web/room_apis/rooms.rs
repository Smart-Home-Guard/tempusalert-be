use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{
    backend_core::{
        features::devices_status_feature::models::{Component, Device},
        models::Room,
    },
    json::Json,
};

use crate::database_client::{init_database, MONGOC};

#[derive(Deserialize, JsonSchema)]
pub struct GetRoomsQuery {
    email: String,
    room_name: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct ResponseDevice {
    id: u32,
    components: Vec<Component>,
}

#[derive(Serialize, JsonSchema)]
pub struct ResponseRoom {
    name: String,
    devices: Vec<ResponseDevice>,
}

#[derive(Serialize, JsonSchema)]
pub enum GetRoomsOfUserResponse {
    GetAllRooms {
        message: String,
        value: Option<Vec<ResponseRoom>>,
    },
    GetOneRoom {
        message: String,
        value: Option<ResponseRoom>,
    },
}

async fn get_rooms_handler(
    headers: HeaderMap,
    Query(GetRoomsQuery { email, room_name }): Query<GetRoomsQuery>,
) -> impl IntoApiResponse {
    if let Some(id) = room_name {
        get_one_handler(headers, email, id).await
    } else {
        get_all_handler(headers, email).await
    }
}

async fn get_all_handler(
    headers: HeaderMap,
    email: String,
) -> (StatusCode, Json<GetRoomsOfUserResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetRoomsOfUserResponse::GetAllRooms {
                message: String::from("Forbidden"),
                value: None,
            }),
        );
    }

    let mongoc = MONGOC.get_or_init(init_database).await;

    let room_coll: Collection<Room> = mongoc.default_database().unwrap().collection("rooms");

    if let Ok(mut room_cursor) = room_coll
        .find(doc! { "owner_name": email.clone() }, None)
        .await
    {
        let mut rooms = vec![];
        while let Ok(true) = room_cursor.advance().await {
            let room = room_cursor.deserialize_current();
            match room {
                Ok(room) => {
                    let devices = futures::future::join_all(room.devices.iter().map(|id| async {
                        let components = get_all_components_by_device(email.clone(), *id).await;
                        ResponseDevice {
                            id: *id,
                            components,
                        }
                    }))
                    .await;

                    rooms.push(ResponseRoom {
                        name: room.name,
                        devices,
                    });
                }
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(GetRoomsOfUserResponse::GetAllRooms {
                            message: format!(
                                "Failed to fetch all rooms for user '{}'",
                                email.clone(),
                            ),
                            value: None,
                        }),
                    );
                }
            }
        }

        return (
            StatusCode::OK,
            Json(GetRoomsOfUserResponse::GetAllRooms {
                message: format!("Fetch all rooms successfully"),
                value: Some(rooms),
            }),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(GetRoomsOfUserResponse::GetAllRooms {
            message: String::from("Internal server error"),
            value: None,
        }),
    )
}

async fn get_one_handler(
    headers: HeaderMap,
    email: String,
    room_name: String,
) -> (StatusCode, Json<GetRoomsOfUserResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetRoomsOfUserResponse::GetOneRoom {
                message: String::from("Forbidden"),
                value: None,
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
        if room_opt.is_none() {
            return (
                StatusCode::BAD_REQUEST,
                Json(GetRoomsOfUserResponse::GetOneRoom {
                    message: format!("No room with name {room_name} for user {email}"),
                    value: None,
                }),
            );
        }

        let room = room_opt.unwrap();
        let devices = futures::future::join_all(room.devices.iter().map(|id| async {
            let components = get_all_components_by_device(email.clone(), *id).await;
            ResponseDevice {
                id: *id,
                components,
            }
        }))
        .await;

        return (
            StatusCode::OK,
            Json(GetRoomsOfUserResponse::GetOneRoom {
                message: format!("Fetch room {room_name} successfully"),
                value: Some({
                    ResponseRoom {
                        name: room.name,
                        devices,
                    }
                }),
            }),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(GetRoomsOfUserResponse::GetOneRoom {
            message: String::from("Internal server error"),
            value: None,
        }),
    )
}

async fn get_all_components_by_device(email: String, id: u32) -> Vec<Component> {
    let mongoc = MONGOC.get_or_init(init_database).await;
    let device_coll: Collection<Device> = mongoc.default_database().unwrap().collection("devices");

    if let Ok(Some(device)) = device_coll
        .find_one(doc! { "owner_name": email, "id": id }, None)
        .await
    {
        device.components
    } else {
        vec![]
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct RoomIdentifier {
    email: String,
    room_name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct NotificationMessageResponse {
    message: String,
}

async fn create_room_handler(
    headers: HeaderMap,
    Json(RoomIdentifier { email, room_name }): Json<RoomIdentifier>,
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

    if room_coll
        .find_one(
            doc! {"name": room_name.clone(), "owner_name": email.clone()},
            None,
        )
        .await
        .unwrap_or(None)
        .is_some()
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(NotificationMessageResponse {
                message: format!("Room '{}' already exists for user '{}'", room_name, email),
            }),
        );
    }

    if let Err(_) = room_coll
        .insert_one(
            Room {
                name: room_name.clone(),
                devices: vec![],
                owner_name: email.clone(),
            },
            None,
        )
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NotificationMessageResponse {
                message: String::from("Failed to create room"),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(NotificationMessageResponse {
            message: format!(
                "Room '{}' created successfully for user '{}'",
                room_name, email
            ),
        }),
    )
}

async fn delete_room_handler(
    headers: HeaderMap,
    Query(RoomIdentifier { email, room_name }): Query<RoomIdentifier>,
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

    if let Ok(None) = room_coll
        .find_one(doc! { "name": &room_name, "owner_name": &email }, None)
        .await
    {
        return (
            StatusCode::NOT_FOUND,
            Json(NotificationMessageResponse {
                message: format!("Room '{}' does not exist for user '{}'", room_name, email),
            }),
        );
    }

    if let Err(_) = room_coll
        .delete_one(doc! { "name": &room_name, "owner_name": &email }, None)
        .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NotificationMessageResponse {
                message: String::from("Failed to delete room"),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(NotificationMessageResponse {
            message: format!(
                "Room '{}' deleted successfully for user '{}'",
                room_name, email
            ),
        }),
    )
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(get_rooms_handler, |op| {
            op.description("Get room by user email")
                .tag("Room")
                .response::<200, Json<GetRoomsOfUserResponse>>()
                .response::<403, Json<GetRoomsOfUserResponse>>()
                .response::<500, Json<GetRoomsOfUserResponse>>()
        })
        .post_with(create_room_handler, |op| {
            op.description("Create new room for specific user")
                .tag("Room")
                .response::<200, Json<NotificationMessageResponse>>()
                .response::<403, Json<NotificationMessageResponse>>()
                .response::<500, Json<NotificationMessageResponse>>()
        })
        .delete_with(delete_room_handler, |op| {
            op.description("Create new room for specific user")
                .tag("Room")
                .response::<200, Json<NotificationMessageResponse>>()
                .response::<403, Json<NotificationMessageResponse>>()
                .response::<500, Json<NotificationMessageResponse>>()
        }),
    )
}
