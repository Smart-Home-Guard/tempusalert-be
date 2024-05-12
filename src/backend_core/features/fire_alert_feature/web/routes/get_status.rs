use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode}, Json,
};
use mongodb::{
    bson::{self, doc, Bson},
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::backend_core::{features::{devices_status_feature::models::Device, fire_alert_feature::models::{FireLog, FireStatus, SensorDataType}}, models::Room};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetStatusQuery {
    email: String,
    component_ids: Option<Vec<usize>>,
    room_name: Option<String>,
    co: Option<()>,
    fire: Option<()>,
    gas: Option<()>,
    heat: Option<()>,
    light: Option<()>,
    button: Option<()>,
    buzzer: Option<()>,
    smoke: Option<()>,
    start_time: Option<i32>,
    end_time: Option<i32>,
}

#[derive(Serialize, JsonSchema)]
pub struct ComponentSafetyStatus {
    id: usize,
    status: FireStatus,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct RoomLogEntryResponse {
    timestamp: usize,
    value: isize,
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
        component_statuses: Option<Vec<ComponentSafetyStatus>>,
    },
    RoomDetailedSafetyStatus {
        message: String,
        component_statuses: Option<Vec<ComponentSafetyStatus>>,
        co_logs: Option<Vec<RoomLogEntryResponse>>,
        heat_logs: Option<Vec<RoomLogEntryResponse>>,
        smoke_logs: Option<Vec<RoomLogEntryResponse>>,
        light_logs: Option<Vec<RoomLogEntryResponse>>,
        buzzer_logs: Option<Vec<RoomLogEntryResponse>>,
        button_logs: Option<Vec<RoomLogEntryResponse>>,
        fire_logs: Option<Vec<RoomLogEntryResponse>>,
        gas_logs: Option<Vec<RoomLogEntryResponse>>,
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
        co,
        fire,
        gas,
        heat,
        light,
        button,
        buzzer,
        smoke,
        start_time,
        end_time,
    }): Query<GetStatusQuery>,
) -> impl IntoApiResponse {
    if room_name.is_some() {
        if vec![co, fire, gas, heat, light, button, buzzer, smoke].into_iter().all(|opt| opt.is_none()) {
            handle_room_status(headers, email, room_name.unwrap()).await
        } else {
            let mut types = vec![];
            co.map(|_| types.push(SensorDataType::CO));
            fire.map(|_| types.push(SensorDataType::Fire));
            gas.map(|_| types.push(SensorDataType::LPG));
            heat.map(|_| types.push(SensorDataType::Heat));
            light.map(|_| types.push(SensorDataType::FireLight));
            button.map(|_| types.push(SensorDataType::FireButton));
            button.map(|_| types.push(SensorDataType::FireBuzzer));
            smoke.map(|_| types.push(SensorDataType::Smoke));

            handle_room_status_of_types(headers, email, room_name.unwrap(), types, start_time, end_time).await
        }
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

async fn handle_room_status_of_types(
    headers: HeaderMap,
    email: String,
    room_name: String,
    types: Vec<SensorDataType>,
    start_time: Option<i32>,
    end_time: Option<i32>,
) -> (StatusCode, Json<GetStatusResponse>) {
    let response = handle_room_status(headers, email.clone(), room_name.clone()).await;
    if let GetStatusResponse::RoomSafetyStatus { message, component_statuses: value } = response.1.0 {
        if value.is_none() || value.as_ref().unwrap().len() == 0 {
            return (
                response.0,
                Json(GetStatusResponse::RoomDetailedSafetyStatus {
                    message,
                    component_statuses: None,
                    co_logs: None,
                    heat_logs: None,
                    smoke_logs: None,
                    light_logs: None,
                    buzzer_logs: None,
                    button_logs: None,
                    fire_logs: None,
                    gas_logs: None,
                }),
            );
        }
        
        let mut co_logs = None;
        let mut heat_logs = None;
        let mut smoke_logs = None;
        let mut light_logs = None;
        let mut buzzer_logs = None;
        let mut button_logs = None;
        let mut fire_logs = None;
        let mut gas_logs = None;

        let fire_coll: Collection<FireLog> = {
            let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
            mongoc.default_database().unwrap().collection("fire_alerts")
        };

        let (component_ids, _) = get_component_ids_by_room(email.clone(), room_name).await.unwrap();

        for typ in types {
            let log_field = match typ {
                SensorDataType::CO => "co_logs",
                SensorDataType::LPG => "gas_logs",
                SensorDataType::Fire => "fire_logs",
                SensorDataType::Heat => "heat_logs",
                SensorDataType::Smoke => "smoke_logs",
                SensorDataType::FireLight => "light_logs",
                SensorDataType::FireButton => "button_logs",
                SensorDataType::FireBuzzer => "buzzer_logs",
            };

            let pipeline = vec![ 
                doc! { "$match": { "owner_name": email.clone() } },
                doc! { "$project": {
                    "_id": 0,
                    log_field: 1,
                } },
                doc! {
                    "$project": {
                        "logs": {
                            "$concatArrays": [format!("${log_field}")],
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
                        "component": { "$in": bson::to_bson(&component_ids).unwrap() },
                    }
                },
                doc! {
                    "$sort": {
                        "timestamp.secs_since_epoch": 1,
                        "timestamp.nanos_since_epoch": 1,
                    }
                },
                doc! {
                    "$match": {
                        "$and": [
                            { "timestamp.secs_since_epoch": { "$gte": start_time.unwrap_or(0) } },
                            { "timestamp.secs_since_epoch": { "$lte": end_time.unwrap_or(i32::MAX) } },
                        ]
                    }
                },
            ];
            
            let mut cursor = fire_coll.aggregate(pipeline, None).await.unwrap();

            let mut res = vec![];
            while let Ok(true) = cursor.advance().await {
                let document = cursor.deserialize_current().unwrap();
                res.push(bson::from_bson::<RoomLogEntryResponse>(document.into()).unwrap());
            }
            
            match typ {
                SensorDataType::CO => { co_logs = Some(res); },
                SensorDataType::LPG => { gas_logs = Some(res); },
                SensorDataType::Fire => { fire_logs = Some(res); },
                SensorDataType::Heat => { heat_logs = Some(res); },
                SensorDataType::Smoke => { smoke_logs = Some(res); },
                SensorDataType::FireLight => { light_logs = Some(res); },
                SensorDataType::FireButton => { button_logs = Some(res); },
                SensorDataType::FireBuzzer => { buzzer_logs = Some(res); },
            };
        }

        return (
            StatusCode::OK,
            Json(GetStatusResponse::RoomDetailedSafetyStatus {
                message,
                component_statuses: value,
                co_logs,
                heat_logs,
                smoke_logs,
                light_logs,
                buzzer_logs,
                button_logs,
                fire_logs,
                gas_logs,
            }),
        );
    }
    panic!("Unreachable");
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
                component_statuses: None,
            }),
        );
    }

    let component_ids = get_component_ids_by_room(email.clone(), room_name).await;
    if component_ids.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(GetStatusResponse::RoomSafetyStatus {
                message: String::from("Room not found"),
                component_statuses: None,
            }),
        );
    }

    let (component_ids, message) = component_ids.unwrap();

    if component_ids.len() == 0 {
        return (
            StatusCode::OK,
            Json(GetStatusResponse::RoomSafetyStatus {
                message,
                component_statuses: Some(vec![]),
            }),
        );
    }

    let statuses = get_component_statuses(email, component_ids).await.map(|inner| inner.into_iter().map(|v| ComponentSafetyStatus { id: v._id, status: v.alert }).collect());

    if statuses.is_none() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetStatusResponse::RoomSafetyStatus {
                message: String::from("Internal server error"),
                component_statuses: None,
            }),
        );
    }

    (
        StatusCode::OK,
        Json(GetStatusResponse::RoomSafetyStatus {
            message: String::from("Successfully fetch all statuses"),
            component_statuses: statuses,
        }),
    )
}

async fn handle_component_statuses(
    headers: HeaderMap,
    email: String,
    component_ids: Vec<usize>,
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

async fn get_component_ids_by_room(email: String, room_name: String) -> Option<(Vec<usize>, String)> {
    let room_coll: Collection<Room> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("rooms")
    };

    let room = room_coll.find_one(doc! { "owner_name": email.clone(), "name": room_name }, None).await.ok().flatten()?;

    let devices = room.devices;

    if devices.len() == 0 {
        return Some((vec![], "Room is empty".to_string()));
    }

    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    let pipeline = vec![
        doc! {
            "$match": {
                "owner_name": email.clone(),
                "id": {
                    "$in": devices.into_iter().map(|d| bson::to_bson(&d).unwrap()).collect::<Vec<Bson>>(),
                },
            }
        },
        doc! {
            "$project": {
                "components": 1,
            }
        }, 
        doc! {
            "$unwind": "$components",
        },
        doc! {
            "$group": {
                "_id": None::<String>,
                "ids": { "$push": "$components.id" },
            }
        },
    ];

    let cursor = device_coll.aggregate(pipeline, None).await;
    if cursor.is_err() {
        return Some((vec![], "Room is empty".to_string()));
    }
    let mut cursor = cursor.unwrap();
    cursor.advance().await.unwrap();
    let document = cursor.deserialize_current().unwrap();
    let component_ids = bson::from_bson(document.get("ids")?.into()).unwrap();

    Some((component_ids, "Fetch all components successfully".to_string()))
}

async fn get_component_statuses(email: String, component_ids: Vec<usize>) -> Option<Vec<ComponentStatusPipelineOutput>> {
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
                "component": { "$in": bson::to_bson(&component_ids).unwrap() },
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
