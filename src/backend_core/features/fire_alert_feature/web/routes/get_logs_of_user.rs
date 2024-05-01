use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{
    bson::{self, doc, Bson},
    options::AggregateOptions,
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::features::fire_alert_feature::{
        fixed_value::MAX_AMOUNT_DOCUMENT_PER_REQUEST,
        models::{FireLog, Pagination, SensorLogData},
    },
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetLogsOfUserQuery {
    email: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    offset: Option<u32>,
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetLogsOfUserResponse {
    fire_logs: Option<Vec<SensorLogData>>,
    smoke_logs: Option<Vec<SensorLogData>>,
    co_logs: Option<Vec<SensorLogData>>,
    heat_logs: Option<Vec<SensorLogData>>,
    button_logs: Option<Vec<SensorLogData>>,
    message: String,
    pagination: Pagination,
}

struct UserLogs {
    fire_logs: Vec<SensorLogData>,
    smoke_logs: Vec<SensorLogData>,
    co_logs: Vec<SensorLogData>,
    heat_logs: Vec<SensorLogData>,
    button_logs: Vec<SensorLogData>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetLogsOfUserQuery {
        email,
        start_time,
        end_time,
        offset,
        limit,
    }): Query<GetLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetLogsOfUserResponse {
                message: String::from("Forbidden"),
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                button_logs: None,
                pagination: Pagination {
                    start_time,
                    end_time,
                    offset,
                    limit,
                },
            }),
        );
    }
    let fire_coll: Collection<FireLog> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("fire_alerts")
    };

    let query_doc = doc! {
        "owner_name": email.clone(),
    };

    let pipeline = vec![
        doc! {
            "$match": query_doc,
        },
        doc! {
            "$project": {
                "fire_logs": {
                    "$filter": {
                        "input": "$fire_logs",
                        "as": "fire_log",
                        "cond": {
                            "$and": [
                                { "$gte": ["$$fire_log.timestamp.secs_since_epoch", start_time.unwrap_or(0)] },
                                { "$lte": ["$$fire_log.timestamp.secs_since_epoch", end_time.unwrap_or(i32::MAX)] }
                            ]
                        }
                    },
                },
                "smoke_logs": {
                    "$filter": {
                        "input": "$smoke_logs",
                        "as": "smoke_log",
                        "cond": {
                            "$and": [
                                { "$gte": ["$$smoke_log.timestamp.secs_since_epoch", start_time.unwrap_or(0)] },
                                { "$lte": ["$$smoke_log.timestamp.secs_since_epoch", end_time.unwrap_or(i32::MAX)] }
                            ]
                        }
                    },
                },
                "co_logs": {
                    "$filter": {
                        "input": "$co_logs",
                        "as": "co_log",
                        "cond": {
                            "$and": [
                                { "$gte": ["$$co_log.timestamp.secs_since_epoch", start_time.unwrap_or(0)] },
                                { "$lte": ["$$co_log.timestamp.secs_since_epoch", end_time.unwrap_or(i32::MAX)] }
                            ]
                        }
                    },
                },
                "heat_logs": {
                    "$filter": {
                        "input": "$heat_logs",
                        "as": "heat_log",
                        "cond": {
                            "$and": [
                                { "$gte": ["$$heat_log.timestamp.secs_since_epoch", start_time.unwrap_or(0)] },
                                { "$lte": ["$$heat_log.timestamp.secs_since_epoch", end_time.unwrap_or(i32::MAX)] }
                            ]
                        }
                    },
                },
                "button_logs": {
                    "$filter": {
                        "input": "$button_logs",
                        "as": "button_log",
                        "cond": {
                            "$and": [
                                { "$gte": ["$$button_log.timestamp.secs_since_epoch", start_time.unwrap_or(0)] },
                                { "$lte": ["$$button_log.timestamp.secs_since_epoch", end_time.unwrap_or(i32::MAX)] }
                            ]
                        }
                    },
                }
            }
        },
        doc! {
            "$project": {
                "fire_logs": { "$slice": ["$fire_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)] },
                "smoke_logs": { "$slice": ["$smoke_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)] },
                "co_logs": { "$slice": ["$co_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)] },
                "heat_logs": { "$slice": ["$heat_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)] },
                "button_logs": { "$slice": ["$button_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)] }
            }
        },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    match fire_coll.aggregate(pipeline, Some(aggregate_options)).await {
        Ok(mut cursor) => {
            let mut user_logs = UserLogs {
                fire_logs: Vec::new(),
                smoke_logs: Vec::new(),
                co_logs: Vec::new(),
                button_logs: Vec::new(),
                heat_logs: Vec::new(),
            };
            while cursor.advance().await.unwrap_or(false) {
                match cursor.deserialize_current() {
                    Ok(document) => {
                        if let Some(Bson::Array(fire_logs_array)) = document.get("fire_logs") {
                            let logs: Vec<SensorLogData> = fire_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            user_logs.fire_logs.extend(logs);
                        }
                        if let Some(Bson::Array(smoke_logs_array)) = document.get("smoke_logs") {
                            let logs: Vec<SensorLogData> = smoke_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            user_logs.smoke_logs.extend(logs);
                        }
                        if let Some(Bson::Array(co_logs_array)) = document.get("co_logs") {
                            let logs: Vec<SensorLogData> = co_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            user_logs.co_logs.extend(logs);
                        }
                        if let Some(Bson::Array(button_logs_array)) = document.get("button_logs") {
                            let logs: Vec<SensorLogData> = button_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            user_logs.button_logs.extend(logs);
                        }
                        if let Some(Bson::Array(heat_logs_array)) = document.get("heat_logs") {
                            let logs: Vec<SensorLogData> = heat_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            user_logs.heat_logs.extend(logs);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error deserializing document: {}", e);
                    }
                }
            }
            if user_logs.fire_logs.is_empty() {
                (
                    StatusCode::OK,
                    Json(GetLogsOfUserResponse {
                        fire_logs: None,
                        smoke_logs: None,
                        co_logs: None,
                        heat_logs: None,
                        button_logs: None,
                        message: format!("Your fire matrix hasn't had any data"),
                        pagination: Pagination {
                            start_time,
                            end_time,
                            offset,
                            limit,
                        },
                    }),
                )
            } else {
                (
                    StatusCode::OK,
                    Json(GetLogsOfUserResponse {
                        fire_logs: Some(user_logs.fire_logs),
                        smoke_logs: Some(user_logs.smoke_logs),
                        co_logs: Some(user_logs.co_logs),
                        heat_logs: Some(user_logs.heat_logs),
                        button_logs: Some(user_logs.button_logs),
                        message: format!("Successfully fetch fire sensor log data"),
                        pagination: Pagination {
                            start_time,
                            end_time,
                            offset,
                            limit,
                        },
                    }),
                )
            }
        }
        Err(e) => {
            eprintln!("Error executing aggregation pipeline: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetLogsOfUserResponse {
                    fire_logs: None,
                    smoke_logs: None,
                    co_logs: None,
                    heat_logs: None,
                    button_logs: None,
                    message: format!("Unexpected error while fetching user log data"),
                    pagination: Pagination {
                        start_time,
                        end_time,
                        offset,
                        limit,
                    },
                }),
            )
        }
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/fire-alert-logs",
        get_with(handler, |op| {
            op.description("Get fire metrics log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetLogsOfUserResponse>>()
                .response::<403, Json<GetLogsOfUserResponse>>()
                .response::<500, Json<GetLogsOfUserResponse>>()
        }),
    )
}
