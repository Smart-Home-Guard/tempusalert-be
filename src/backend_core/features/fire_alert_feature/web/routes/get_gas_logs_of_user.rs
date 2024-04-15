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
pub struct GetGasLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetGasLogsOfUserResponse {
    message: String,
    gas_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetGasLogsOfUserQuery { email }): Query<GetGasLogsOfUserQuery>,
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
            Json(GetGasLogsOfUserResponse {
                message: format!("Successfully fetch gas log data"),
                gas_logs: Some(fire_log.lpg_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetGasLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                gas_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetGasLogsOfUserResponse {
                message: format!("Unexpected error while fetching gas log data"),
                gas_logs: None,
            }),
        ),
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
