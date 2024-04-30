use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::{self, doc, Bson}, options::AggregateOptions, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::features::fire_alert_feature::{fixed_value::MAX_AMOUNT_DOCUMENT_PER_REQUEST, models::{FireLog, Pagination, SensorLogData}},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetButtonLogsOfUserQuery {
    email: String,
    start_time: Option<i32>,
    end_time: Option<i32>,
    offset: Option<u32>,
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct GetButtonLogsOfUserResponse {
    message: String,
    button_logs: Option<Vec<SensorLogData>>,
    pagination: Pagination,
}

async fn handler(
    headers: HeaderMap,
    Query(GetButtonLogsOfUserQuery {
        email,
        start_time,
        end_time,
        offset,
        limit,
    }): Query<GetButtonLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetButtonLogsOfUserResponse {
                message: String::from("Forbidden"),
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
            "$unwind": "$button_logs"
        },
        doc! {
            "$sort": {
                "button_logs.timestamp.secs_since_epoch": -1,
                "button_logs.timestamp.nanos_since_epoch": -1
            }
        },
        doc! {
            "$match": {
                "$and": [
                    { "button_logs.timestamp.secs_since_epoch": { "$gt": start_time.unwrap_or(0) } },
                    { "button_logs.timestamp.secs_since_epoch": { "$lt": end_time.unwrap_or(i32::MAX) } }
            ]
        }
            },
        doc! {
            "$group": {
                "_id": "$_id",
                "button_logs": { "$push": "$button_logs" }
            }
        },
        doc! {
            "$project": {
                "button_logs": {
                    "$slice": ["$button_logs", offset.unwrap_or(0), limit.unwrap_or(MAX_AMOUNT_DOCUMENT_PER_REQUEST)],
                }
            }
        },
    ];

    let aggregate_options = AggregateOptions::builder().build();

    match fire_coll.aggregate(pipeline, Some(aggregate_options)).await {
        Ok(mut cursor) => {
            let mut button_logs = Vec::new();
            while cursor.advance().await.unwrap_or(false) {
                match cursor.deserialize_current() {
                    Ok(document) => {
                        if let Some(Bson::Array(button_logs_array)) = document.get("button_logs") {
                            let logs: Vec<SensorLogData> = button_logs_array
                                .iter()
                                .filter_map(|log| {
                                    bson::from_bson::<SensorLogData>(log.clone()).ok()
                                })
                                .collect();
                            button_logs.extend(logs);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error deserializing document: {}", e);
                    }
                }
            }
            if button_logs.is_empty() {
                (
                    StatusCode::OK,
                    Json(GetButtonLogsOfUserResponse {
                        message: format!("Your fire matrix hasn't had any data"),
                        button_logs: None,
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
                    Json(GetButtonLogsOfUserResponse {
                        message: format!("Successfully fetched fire log data"),
                        button_logs: Some(button_logs),
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
                Json(GetButtonLogsOfUserResponse {
                    message: format!("Unexpected error while fetching fire log data"),
                    button_logs: None,
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
        "/button-logs",
        get_with(handler, |op| {
            op.description("Get button log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetButtonLogsOfUserResponse>>()
                .response::<403, Json<GetButtonLogsOfUserResponse>>()
                .response::<500, Json<GetButtonLogsOfUserResponse>>()
        }),
    )
}
