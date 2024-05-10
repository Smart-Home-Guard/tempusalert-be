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
    backend_core::features::fire_alert_feature::models::{FireLog, FireStatus},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetStatusQuery {
    email: String,
    component_ids: Option<Vec<String>>,
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
    }): Query<GetStatusQuery>,
) -> impl IntoApiResponse {
    if component_ids.is_some() {
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

    let cursor = fire_coll.aggregate(pipeline, None).await;
    if !cursor.is_ok() {
        return (
            StatusCode::BAD_REQUEST,
            Json(GetStatusResponse::ComponentSafetyStatuses {
                message: String::from("Bad request"),
                value: None,
            }),
        );
    }

    let mut cursor = cursor.unwrap();
    if !cursor.advance().await.is_ok() {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetStatusResponse::ComponentSafetyStatuses {
                message: String::from("Internal server error"),
                value: None,
            }),
        );
    }

    let document = cursor.deserialize_current().unwrap();
    let statuses = bson::from_bson::<Vec<ComponentStatusPipelineOutput>>(document.into()).ok().map(|inner| inner.into_iter().map(|v| ComponentSafetyStatus { id: v._id, status: v.alert }).collect());
    (
        StatusCode::OK,
        Json(GetStatusResponse::ComponentSafetyStatuses {
            message: String::from("Get fire status successfully"),
            value: statuses, 
        })
    )

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
