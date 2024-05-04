use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{
    bson::{self, doc, Bson},
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{
    backend_core::{features::devices_status_feature::models::Component, models::Room},
    json::Json,
};

use crate::database_client::{init_database, MONGOC};

#[derive(Deserialize, JsonSchema)]
pub struct GetRoomsQuery {
    email: String,
    room_id: Option<String>,
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

async fn handler(
    headers: HeaderMap,
    Query(GetRoomsQuery { email, room_id }): Query<GetRoomsQuery>,
) -> impl IntoApiResponse {
    if let Some(id) = room_id {
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

    let query_doc = doc! {
        "owner_name": email.clone(),
    };

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
    room_id: String,
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

    let query_doc = doc! {
        "owner_name": email.clone(),
    };

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(GetRoomsOfUserResponse::GetOneRoom {
            message: String::from("Internal server error"),
            value: None,
        }),
    )
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(handler, |op| {
            op.description("Get room by user email")
                .tag("Room")
                .response::<200, Json<GetRoomsOfUserResponse>>()
                .response::<403, Json<GetRoomsOfUserResponse>>()
                .response::<500, Json<GetRoomsOfUserResponse>>()
        }),
    )
}
