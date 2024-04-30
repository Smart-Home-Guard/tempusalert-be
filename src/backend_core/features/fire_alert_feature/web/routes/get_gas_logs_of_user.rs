use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{
    bson::{self, doc, Bson}, options::AggregateOptions, Collection
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::features::fire_alert_feature::{fixed_value::MAX_AMOUNT_DOCUMENT_PER_REQUEST, models::{FireLog, Pagination, SensorLogData}},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetGasLogsOfUserQuery {
    email: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    offset: Option<u32>,
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetGasLogsOfUserResponse {
    message: String,
    gas_logs: Option<Vec<SensorLogData>>,
    pagination: Pagination,
}

async fn handler(
    headers: HeaderMap,
    Query(GetGasLogsOfUserQuery {
        email,
        start_time,
        end_time,
        offset,
        limit,
    }): Query<GetGasLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetGasLogsOfUserResponse {
                message: String::from("Forbidden"),
                gas_logs: None,
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
            "$unwind": "$lpg_logs"
        },
        doc! {
            "$sort": {
                "lpg_logs.timestamp.secs_since_epoch": -1,
                "lpg_logs.timestamp.nanos_since_epoch": -1
            }
        },
        doc! {
            "$match": {
                "$and": [
                    { "lpg_logs.timestamp.secs_since_epoch": { "$gt": start_time.unwrap_or(0) } },
                    { "lpg_logs.timestamp.secs_since_epoch": { "$lt": end_time.unwrap_or(i32::MAX) } }
            ]
        }
            },
        doc! {
            "$group": {
                "_id": "$_id",
                "lpg_logs": { "$push": "$lpg_logs" }
            }
        },
        doc! {
            "$project": {
                "lpg_logs": {
                    "$slice": ["$lpg_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)],
                }
            }
        },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    match fire_coll.aggregate(pipeline, Some(aggregate_options)).await {
        Ok(mut cursor) => {
            let mut lpg_logs = Vec::new();
            while cursor.advance().await.unwrap_or(false) {
                match cursor.deserialize_current() {
                    Ok(document) => {
                        if let Some(Bson::Array(lpg_logs_array)) = document.get("lpg_logs") {
                            let logs: Vec<SensorLogData> = lpg_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            lpg_logs.extend(logs);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error deserializing document: {}", e);
                    }
                }
            }
            if lpg_logs.is_empty() {
                (
                    StatusCode::OK,
                    Json(GetGasLogsOfUserResponse {
                        message: format!("Your fire matrix hasn't had any data"),
                        gas_logs: None,
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
                    Json(GetGasLogsOfUserResponse {
                        message: format!("Successfully fetch smoke log data"),
                        gas_logs: Some(lpg_logs),
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
                Json(GetGasLogsOfUserResponse {
                    message: format!("Unexpected error while fetching smoke log data"),
                    gas_logs: None,
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
        "/gas-logs",
        get_with(handler, |op| {
            op.description("Get gas log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetGasLogsOfUserResponse>>()
                .response::<403, Json<GetGasLogsOfUserResponse>>()
                .response::<500, Json<GetGasLogsOfUserResponse>>()
        }),
    )
}
