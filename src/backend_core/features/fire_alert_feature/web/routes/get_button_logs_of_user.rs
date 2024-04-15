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
pub struct GetButtonLogsOfUserQuery {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct GetButtonLogsOfUserResponse {
    message: String,
    button_logs: Option<Vec<SensorLogData>>,
}

async fn handler(
    headers: HeaderMap,
    Query(GetButtonLogsOfUserQuery { email }): Query<GetButtonLogsOfUserQuery>,
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
            Json(GetButtonLogsOfUserResponse {
                message: format!("Successfully fetch button log data"),
                button_logs: Some(fire_log.button_logs),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(GetButtonLogsOfUserResponse {
                message: format!("Your fire matrix haven't had any data"),
                button_logs: None,
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GetButtonLogsOfUserResponse {
                message: format!("Unexpected error while fetching button log data"),
                button_logs: None,
            }),
        ),
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
