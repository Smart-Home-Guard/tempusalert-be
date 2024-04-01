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
pub struct GetLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetLogsOfUserResponse {
    fire_logs: Option<Vec<SensorLogData>>,
    smoke_logs: Option<Vec<SensorLogData>>,
    co_logs: Option<Vec<SensorLogData>>,
    heat_logs: Option<Vec<SensorLogData>>,
    button_logs: Option<Vec<SensorLogData>>,
    message: String,
}

async fn handler(
    headers: HeaderMap,
    Query(GetLogsOfUserQuery { email }): Query<GetLogsOfUserQuery>,
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
            Json(GetLogsOfUserResponse {
                fire_logs: Some(fire_log.fire_logs),
                smoke_logs: Some(fire_log.smoke_logs),
                co_logs: Some(fire_log.co_logs),
                heat_logs: Some(fire_log.heat_logs),
                button_logs: Some(fire_log.button_logs),
                message: format!("Successfully fetch fire sensor log data"),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetLogsOfUserResponse {
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                button_logs: None,
                message: format!("Your fire matrix haven't had any data"),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetLogsOfUserResponse {
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                button_logs: None,
                message: format!("Unexpected error while fetching fire log data"),
            }),
        ),
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
