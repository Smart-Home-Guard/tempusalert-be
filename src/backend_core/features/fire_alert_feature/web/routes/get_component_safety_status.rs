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
    backend_core::features::{
        devices_status_feature::{
            iot::mqtt_messages::ComponentType,
            models::{Component, Device}
        },
        fire_alert_feature::models::{FireLog, FireStatus, SensorLogData},
    },
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetComponentSafetyStatusQuery {
    email: String,
    component_id: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetComponentSafetyStatusResponse {
    message: String,
    status: Option<FireStatus>
}

async fn handler(
    headers: HeaderMap,
    Query(GetComponentSafetyStatusQuery {
        email,
        component_id,
    }): Query<GetComponentSafetyStatusQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetComponentSafetyStatusResponse {
                message: String::from("Forbidden"),
                status: None,
            }),
        );
    }

    let fire_device_type = get_fire_component_type(email.clone(), component_id.clone()).await;

    if fire_device_type.is_none() {
        return (
            StatusCode::NOT_FOUND,
            Json(GetComponentSafetyStatusResponse {
                message: String::from("Component not found"),
                status: None,
            }),
        );
    }
    
    let fire_device_type = fire_device_type.unwrap();

    let fire_coll: Collection<FireLog> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("fire_alerts")
    };

   let log_field = get_kind_log_field(fire_device_type); 

   let pipeline = vec![
        doc! { "$match": { "owner_name": email.clone() } },
        doc! { "$unwind": format!("${log_field}") },
        doc! { "$match": { format!("{log_field}"): { "id": component_id } } },
        doc! {
            "$sort": {
                format!("{log_field}.timestamp.secs_since_epoch"): -1,
                format!("{log_field}.timestamp.nanos_since_epoch"): -1,
            }
        },
        doc! { "$limit": 1 },
   ];

    let cursor = fire_coll.aggregate(pipeline, None).await;
    if !cursor.is_ok() {
        return (
            StatusCode::NOT_FOUND,
            Json(GetComponentSafetyStatusResponse {
                message: String::from("Component not found"),
                status: None,
            }),
        );
    }

    let mut cursor = cursor.unwrap();
    if !cursor.advance().await.is_ok() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetComponentSafetyStatusResponse {
                message: String::from("Internal server error"),
                status: None,
            }),
        );
    }

    let document = cursor.deserialize_current().unwrap();
    let status = bson::from_bson::<SensorLogData>(document.get(log_field).unwrap().into()).map(|s| s.alert).ok();
    (
        StatusCode::OK,
        Json(GetComponentSafetyStatusResponse {
            message: String::from("Get fire status successfully"),
            status, 
        })
    )
}

async fn get_fire_component_type(email: String, component_id: String) -> Option<ComponentType> {
    let pipeline = vec![
        doc! { "$match": { "owner_email": email } },
        doc! { "$unwind": "$components" },
        doc! { "$match": { "components.id": component_id } },
    ];

    let device_coll: Collection<Device> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("devices")
    };

    let mut cursor = device_coll.aggregate(pipeline, None).await.ok()?;
    cursor.advance().await.ok()?;
    let document = cursor.deserialize_current().ok()?;
    let component = bson::from_bson::<Component>(document.get("components").unwrap().into()).ok()?;

    match component.kind {
        | ComponentType::CO
        | ComponentType::LPG
        | ComponentType::Heat
        | ComponentType::Fire
        | ComponentType::Smoke
        | ComponentType::FireLight
        | ComponentType::FireButton
        | ComponentType::FireBuzzer
        => Some(component.kind),

        _ => None,
    }
}

fn get_kind_log_field(kind: ComponentType) -> String {
    String::from(match kind {
        ComponentType::CO => "co_logs",
        ComponentType::LPG => "lpg_logs",
        ComponentType::Heat => "heat_logs",
        ComponentType::Fire => "fire_logs",
        ComponentType::Smoke => "smoke_logs",
        ComponentType::FireLight => "light_logs",
        ComponentType::FireButton => "button_logs",
        ComponentType::FireBuzzer => "buzzer_logs",
        _ => panic!("Unreachable"),
    })
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/status",
        get_with(handler, |op| {
            op.description("Get fire-device status")
                .tag("Fire alert")
                .response::<200, Json<GetComponentSafetyStatusResponse>>()
                .response::<403, Json<GetComponentSafetyStatusResponse>>()
                .response::<500, Json<GetComponentSafetyStatusResponse>>()
        }),
    )
}
