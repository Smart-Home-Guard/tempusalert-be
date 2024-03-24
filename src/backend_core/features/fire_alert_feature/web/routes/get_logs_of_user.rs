use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformParameter,
};
use axum::{
    extract::Path,
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
pub struct Params {
    email: String,
}

#[derive(Serialize, JsonSchema)]
pub struct Response {
    fire_logs: Option<Vec<SensorLogData>>,
    smoke_logs: Option<Vec<SensorLogData>>,
    co_logs: Option<Vec<SensorLogData>>,
    heat_logs: Option<Vec<SensorLogData>>,
    fire_button_logs: Option<Vec<SensorLogData>>,
    message: String,
}

async fn handler(headers: HeaderMap, Path(Params { email }): Path<Params>) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(Response {
                message: String::from("Forbidden"),
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                fire_button_logs: None,
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
            Json(Response {
                fire_logs: Some(fire_log.fire_logs),
                smoke_logs: Some(fire_log.smoke_logs),
                co_logs: Some(fire_log.co_logs),
                heat_logs: Some(fire_log.heat_logs),
                fire_button_logs: Some(fire_log.fire_button_logs),
                message: format!("Successfully fetch fire sensor log data"),
            }),
        ),
        Ok(None) => (
            StatusCode::OK,
            Json(Response {
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                fire_button_logs: None,
                message: format!("Your fire matrix haven't had any data"),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(Response {
                fire_logs: None,
                smoke_logs: None,
                co_logs: None,
                heat_logs: None,
                fire_button_logs: None,
                message: format!("Unexpected error while fetching fire log data"),
            }),
        ),
    }
}

pub fn routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/:email/logs",
        get_with(handler, |op| {
            op.description("Get fire metrics log by user email")
                .tag("Fire alert")
                .parameter("email", |op: TransformParameter<String>| {
                    op.description("The registered email")
                })
                .response::<200, Json<Response>>()
                .response::<403, Json<Response>>()
                .response::<500, Json<Response>>()
        }),
    )
}
