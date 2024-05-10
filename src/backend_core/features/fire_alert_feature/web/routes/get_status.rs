use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{
    bson::{self, doc},
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::{features::{devices_status_feature::models::Device, fire_alert_feature::models::{FireLog, FireStatus}}, models::Room},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetStatusQuery {
    email: String,
    component_ids: Option<Vec<String>>,
    room_name: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct ComponentSafetyStatus {
    id: usize,
    status: FireStatus,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum GetStatusResponse {
    ComponentSafetyStatuses {
        message: String,
        value: Option<Vec<ComponentSafetyStatus>>,
    },
    RoomSafetyStatus {
        message: String,
        value: Option<Vec<ComponentSafetyStatus>>,
    },
    Unrecognized {
        message: String,
        value: Option<()>,
    }
}

#[derive(Deserialize)]
struct ComponentStatusPipelineOutput {
    _id: usize,
    alert: FireStatus,
}

async fn handler(
    headers: HeaderMap,
    Query(GetStatusQuery {
        email,
        component_ids,
        room_name,
    }): Query<GetStatusQuery>,
) -> impl IntoApiResponse {
    if room_name.is_some() {
        handle_room_status(headers, email, room_name.unwrap()).await
    } else if component_ids.is_some() {
        handle_component_statuses(headers, email, component_ids.unwrap()).await
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(GetStatusResponse::Unrecognized {
                message: String::from("Bad request"),
                value: None,
            }),
        )
    }
}

async fn handle_room_status(
    headers: HeaderMap,
    email: String,
    room_name: String,
) -> (StatusCode, Json<GetStatusResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetStatusResponse::RoomSafetyStatus {
                message: String::from("Forbidden"),
                value: None,
            }),
        );
    }

    let component_ids = get_component_ids_by_room(email.clone(), room_name).await;

    if component_ids.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(GetStatusResponse::RoomSafetyStatus {
                message: String::from("Room not found"),
                value: None,
            }),
        );
    }

    let component_ids = component_ids.unwrap();

    let statuses = get_component_statuses(email, component_ids).await.map(|inner| inner.into_iter().map(|v| ComponentSafetyStatus { id: v._id, status: v.alert }).collect());

    if statuses.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetStatusResponse::RoomSafetyStatus {
                message: String::from("Internal server error"),
                value: None,
            }),
        );
    }

    (
        StatusCode::OK,
        Json(GetStatusResponse::RoomSafetyStatus {
            message: String::from("Successfully fetch all statuses"),
            value: statuses,
        }),
    )
}

async fn handle_component_statuses(
    headers: HeaderMap,
    email: String,
    component_ids: Vec<String>,
) -> (StatusCode, Json<GetStatusResponse>) {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetStatusResponse::ComponentSafetyStatuses {
                message: String::from("Forbidden"),
                value: None,
            }),
        );
    }

    let statuses = get_component_statuses(email, component_ids).await.map(|inner| inner.into_iter().map(|v| ComponentSafetyStatus { id: v._id, status: v.alert }).collect());
    (
        StatusCode::OK,
        Json(GetStatusResponse::ComponentSafetyStatuses {
            message: String::from("Get fire status successfully"),
            value: statuses, 
        })
    )
}

async fn get_component_ids_by_room(email: String, room_name: String) -> Option<Vec<String>> {
    let room_coll: Collection<Room> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("rooms")
    };

    let room = room_coll.find_one(doc! { "owner_name": email.clone(), "name": room_name }, None).await.ok().flatten()?;

    let devices = room.devices;

    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };
    
    let pipeline = vec![
        doc! {
            "$match": {
                "owner_name": email.clone(),
                "id": {
                    "$in": bson::to_document(&devices).ok()?,
                },
            }
        },
       doc! {
            "$project": {
                "components": 1,
            }
        }, 
        doc! {
            "$unwind": "components",
        },
        doc! {
            "$group": {
                "_id": None::<String>,
                "ids": { "$push": "$components.id" },
            }
        },
        doc! {
            "$replaceRoot": {
                "newRoot": "$ids",
            }
        },
    ];

    let mut cursor = device_coll.aggregate(pipeline, None).await.ok()?;
    let mut component_ids = vec![];
    while cursor.advance().await.ok()? {
        let document = cursor.deserialize_current().ok()?;
        component_ids.push(bson::from_bson(document.into()).ok()?);
    }

    Some(component_ids)
}

async fn get_component_statuses(email: String, component_ids: Vec<String>) -> Option<Vec<ComponentStatusPipelineOutput>> {
    let fire_coll: Collection<FireLog> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("fire_alerts")
    };

    let pipeline = vec![ 
        doc! { "$match": { "owner_name": email.clone() } },
        doc! { "$project": {
            "_id": 0,
            "co_logs": 1,
            "buzzer_logs": 1,
            "fire_logs": 1,
            "gas_logs": 1,
            "heat_logs": 1,
            "light_logs": 1,
            "smoke_logs": 1,
            "button_logs": 1,
        } },
        doc! {
            "$project": {
                "logs": {
                    "$concatArrays": ["$co_logs", "$buzzer_logs", "$gas_logs", "$heat_logs", "$light_logs", "$smoke_logs", "$button_logs"],
                }
            }
        },
        doc! {
            "$replaceRoot": {
                "newRoot": "$logs",
            }
        },
        doc! {
            "$match": {
                "component": { "$in": component_ids },
            }
        },
        doc! {
            "$sort": {
                "timestamp.secs_since_epoch": -1,
                "timestamp.nanos_since_epoch": -1,
            }
        },
        doc! {
            "$group": {
                "_id": "$component",
                "alert": { "$first": "$alert" },
            }
        }, 
    ];

    let mut cursor = fire_coll.aggregate(pipeline, None).await.ok()?;
    let mut res = vec![];
    while cursor.advance().await.ok()? {
        let document = cursor.deserialize_current().ok()?;
        res.push(bson::from_bson(document.into()).ok()?);
    }

    Some(res)
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/status",
        get_with(handler, |op| {
            op.description("Get fire-device status")
                .tag("Fire alert")
                .response::<200, Json<GetStatusResponse>>()
                .response::<403, Json<GetStatusResponse>>()
                .response::<500, Json<GetStatusResponse>>()
        }),
    )
}
