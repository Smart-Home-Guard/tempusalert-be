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
pub struct GetSmokeLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetSmokeLogsOfUserResponse {
    message: String,
    smoke_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetSmokeLogsOfUserQuery { email }): Query<GetSmokeLogsOfUserQuery>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(GetSmokeLogsOfUserResponse {
                message: String::from("Forbidden"),
                smoke_logs: None,
            }),
        );
    }
    let fire_coll: Collection<FireLog> = {
        let mongoc = unsafe { MONGOC.as_ref().clone().unwrap().lock() }.await;
        mongoc.default_database().unwrap().collection("fire_alerts")
    };

    match fire_coll
        .find_one(doc! {"owner_name": email.clone() }, None)
        .await
    {
        Ok(Some(fire_log)) => (
            StatusCode::OK,
            Json(GetSmokeLogsOfUserResponse {
                message: format!("Successfully fetch smoke log data"),
                smoke_logs: Some(fire_log.smoke_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetSmokeLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                smoke_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetSmokeLogsOfUserResponse {
                message: format!("Unexpected error while fetching smoke log data"),
                smoke_logs: None,
            }),
        ),
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/smoke-logs",
        get_with(handler, |op| {
            op.description("Get smoke log by user email")
                .tag("Fire alert")
                .response::<200, Json<GetSmokeLogsOfUserResponse>>()
                .response::<403, Json<GetSmokeLogsOfUserResponse>>()
                .response::<500, Json<GetSmokeLogsOfUserResponse>>()
        }),
    )
}
