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
pub struct GetFireLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetFireLogsOfUserResponse {
    message: String,
    fire_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetFireLogsOfUserQuery { email }): Query<GetFireLogsOfUserQuery>,
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
            Json(GetFireLogsOfUserResponse {
                message: format!("Successfully fetch fire log data"),
                fire_logs: Some(fire_log.fire_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetFireLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                fire_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetFireLogsOfUserResponse {
                message: format!("Unexpected error while fetching fire log data"),
                fire_logs: None,
            }),
        ),
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
