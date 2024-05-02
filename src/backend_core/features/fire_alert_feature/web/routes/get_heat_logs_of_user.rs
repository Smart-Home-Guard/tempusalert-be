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
pub struct GetHeatLogsOfUserQuery {
    email: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    offset: Option<u32>,
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetHeatLogsOfUserResponse {
    message: String,
    heat_logs: Option<Vec<SensorLogData>>,
    pagination: Pagination,
}

async fn handler(
    headers: HeaderMap,
    Query(GetHeatLogsOfUserQuery {
        email,
        start_time,
        end_time,
        offset,
        limit,
    }): Query<GetHeatLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetHeatLogsOfUserResponse {
                message: String::from("Forbidden"),
                heat_logs: None,
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
            "$unwind": "$heat_logs"
        },
        doc! {
            "$sort": {
                "heat_logs.timestamp.secs_since_epoch": -1,
                "heat_logs.timestamp.nanos_since_epoch": -1
            }
        },
        doc! {
            "$match": {
                "$and": [
                    { "heat_logs.timestamp.secs_since_epoch": { "$gte": start_time.unwrap_or(0) } },
                    { "heat_logs.timestamp.secs_since_epoch": { "$lte": end_time.unwrap_or(i32::MAX) } }
            ]
            }
        },
        doc! {
            "$group": {
                "_id": "$_id",
                "heat_logs": { "$push": "$heat_logs" }
            }
        },
        doc! {
            "$project": {
                "heat_logs": {
                    "$slice": ["$heat_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)],
                }
            }
        },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    match fire_coll.aggregate(pipeline, Some(aggregate_options)).await {
        Ok(mut cursor) => {
            let mut heat_logs = Vec::new();
            while cursor.advance().await.unwrap_or(false) {
                match cursor.deserialize_current() {
                    Ok(document) => {
                        if let Some(Bson::Array(heat_logs_array)) = document.get("heat_logs") {
                            let logs: Vec<SensorLogData> = heat_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            heat_logs.extend(logs);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error deserializing document: {}", e);
                    }
                }
            }
            if heat_logs.is_empty() {
                (
                    StatusCode::OK,
                    Json(GetHeatLogsOfUserResponse {
                        message: format!("Your fire matrix hasn't had any data"),
                        heat_logs: None,
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
                    Json(GetHeatLogsOfUserResponse {
                        message: format!("Successfully fetch smoke log data"),
                        heat_logs: Some(heat_logs),
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
                Json(GetHeatLogsOfUserResponse {
                    message: format!("Unexpected error while fetching smoke log data"),
                    heat_logs: None,
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
        "/heat-logs",
        get_with(handler, |op| {
            op.description("Get heat log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetHeatLogsOfUserResponse>>()
                .response::<403, Json<GetHeatLogsOfUserResponse>>()
                .response::<500, Json<GetHeatLogsOfUserResponse>>()
        }),
    )
}
