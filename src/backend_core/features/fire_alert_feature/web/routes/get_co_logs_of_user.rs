use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    backend_core::features::fire_alert_feature::models::{FireLog, SensorLogData},
    json::Json,
};

use super::MONGOC;

#[derive(Deserialize, JsonSchema)]
pub struct GetCOLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetCOLogsOfUserResponse {
    message: String,
    co_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetCOLogsOfUserQuery { email }): Query<GetCOLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetCOLogsOfUserResponse {
                message: String::from("Forbidden"),
                co_logs: None,
            }),
        );
    }
    let fire_coll: Collection<FireLog> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("fire")
    };

    match fire_coll
        .find_one(doc! {"owner_name": email.clone() }, None)
        .await
    {
        Ok(Some(fire_log)) => (
            StatusCode::OK,
            Json(GetCOLogsOfUserResponse {
                message: format!("Successfully fetch co log data"),
                co_logs: Some(fire_log.co_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetCOLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                co_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetCOLogsOfUserResponse {
                message: format!("Unexpected error while fetching co log data"),
                co_logs: None,
            }),
        ),
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/co-logs",
        get_with(handler, |op| {
            op.description("Get co log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetCOLogsOfUserResponse>>()
                .response::<403, Json<GetCOLogsOfUserResponse>>()
                .response::<500, Json<GetCOLogsOfUserResponse>>()
        }),
    )
}
