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
pub struct GetHeatLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetHeatLogsOfUserResponse {
    message: String,
    heat_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetHeatLogsOfUserQuery { email }): Query<GetHeatLogsOfUserQuery>,
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
            Json(GetHeatLogsOfUserResponse {
                message: format!("Successfully fetch heat log data"),
                heat_logs: Some(fire_log.heat_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetHeatLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                heat_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetHeatLogsOfUserResponse {
                message: format!("Unexpected error while fetching heat log data"),
                heat_logs: None,
            }),
        ),
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
