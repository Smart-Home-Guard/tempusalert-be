use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{
    bson::{self, doc, Bson},
    options::{AggregateOptions, FindOptions},
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::features::fire_alert_feature::models::{FireLog, Pagination, SensorLogData},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetFireLogsOfUserQuery {
    email: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    offset: Option<u32>,
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetFireLogsOfUserResponse {
    message: String,
    fire_logs: Option<Vec<SensorLogData>>,
    pagination: Pagination,
}

async fn handler(
    headers: HeaderMap,
    Query(GetFireLogsOfUserQuery {
        email,
        start_time,
        end_time,
        offset,
        limit,
    }): Query<GetFireLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetFireLogsOfUserResponse {
                message: String::from("Forbidden"),
                fire_logs: None,
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
                "fire_logs": 1
            }
        },
        doc! {
            "$unwind": "$fire_logs"
        },
        doc! {
            "$sort": {
                "fire_logs.timestamp.secs_since_epoch": -1,
                "fire_logs.timestamp.nanos_since_epoch": -1
            }
        },
        doc! {
            "$match": {
                "fire_logs.timestamp.secs_since_epoch": { "$gt": start_time.unwrap_or(0) },
                "fire_logs.timestamp.secs_since_epoch": { "$lt": end_time.unwrap() },
            }
        },
        doc! {
            "$group": {
                "_id": "$_id",
                "fire_logs": { "$push": "$fire_logs" }
            }
        },
        doc! {
            "$project": {
                "fire_logs": {
                    "$slice": ["$fire_logs", offset.unwrap_or(0), limit.unwrap()],
                }
            }
        },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    match fire_coll.aggregate(pipeline, Some(aggregate_options)).await {
        Ok(mut cursor) => {
            let mut fire_logs = Vec::new();
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
                            fire_logs.extend(logs);
                        }
                    }
                    Err(e) => {
                        println!("Error deserializing document: {}", e);
                    }
                }
            }
            if fire_logs.is_empty() {
                (
                    StatusCode::OK,
                    Json(GetFireLogsOfUserResponse {
                        message: format!("Your fire matrix hasn't had any data"),
                        fire_logs: None,
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
                    Json(GetFireLogsOfUserResponse {
                        message: format!("Successfully fetched fire log data"),
                        fire_logs: Some(fire_logs),
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
            println!("Error executing aggregation pipeline: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GetFireLogsOfUserResponse {
                    message: format!("Unexpected error while fetching fire log data"),
                    fire_logs: None,
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
        "/fire-logs",
        get_with(handler, |op| {
            op.description("Get fire log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetFireLogsOfUserResponse>>()
                .response::<403, Json<GetFireLogsOfUserResponse>>()
                .response::<500, Json<GetFireLogsOfUserResponse>>()
        }),
    )
}
