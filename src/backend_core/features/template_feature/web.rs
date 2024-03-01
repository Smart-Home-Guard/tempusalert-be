use axum::async_trait;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use std::result::Result;
use std::sync::Arc;
use utoipa::openapi::{path::OperationBuilder, PathItem, PathItemType};
use utoipa::{ToResponse, ToSchema};

use super::super::{WebFeature, SwaggerMeta};
use super::IotExampleFeature;
use crate::backend_core::features::IotFeature;
use crate::errors::AppError;
use crate::message::{IotNotification, WebNotification};

#[derive(Serialize, ToSchema, ToResponse)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebExampleFeature;

impl WebExampleFeature {
    pub async fn health_check() -> Result<Json<GenericResponse>, AppError> {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };
        Ok(Json(response_json))
    }
}

#[async_trait]
impl WebFeature for WebExampleFeature {
    fn create(web_rx: Arc<Mutex<Receiver<IotNotification>>>, web_tx: &mut Sender<WebNotification>) -> Self {
        WebExampleFeature
    }

    fn set_receiver(&mut self, iot_feat: Arc<dyn IotFeature>) {
        
    }

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    async fn send(&mut self, notif: WebNotification) {

    }

    async fn recv(&mut self) -> IotNotification {
        IotNotification::None
    }


    fn create_router(&mut self) -> Router {
        Router::new().route("/api/health_check", get(WebExampleFeature::health_check))
    }

    fn create_swagger(&mut self) -> SwaggerMeta {
        SwaggerMeta {
            key: String::from("/api/health_check"),
            value: PathItem::new(
                PathItemType::Get,
                OperationBuilder::new()
                    .responses(
                        utoipa::openapi::ResponsesBuilder::new()
                            .response(
                                "200",
                                utoipa::openapi::ResponseBuilder::new()
                                    .description("Pet found succesfully")
                                    .content(
                                        "application/json",
                                        utoipa::openapi::Content::new(
                                            utoipa::openapi::Ref::from_response_name(
                                                "GenericResponse",
                                            ),
                                        ),
                                    ),
                            )
                            .response("404", utoipa::openapi::Response::new("Pet was not found")),
                    )
                    .operation_id(Some("get_pet_by_id"))
                    .deprecated(Some(utoipa::openapi::Deprecated::False))
                    .summary(Some("Get pet by id"))
                    .description(Some(
                        "Get pet by id\n\nGet pet from database by pet database id\n",
                    ))
                    .tag("health_check_api"),
            ),
        }
    }

    async fn process_iot_notification(&mut self, notification: IotNotification) {
        
    }
}
