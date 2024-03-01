use axum::async_trait;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::backend_core::features::{IotFeature, WebFeature};
use crate::errors::AppError;
use crate::message::{IotNotification, WebNotification};

#[derive(Serialize)]
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
    fn create(
        web_rx: Arc<Mutex<Receiver<IotNotification>>>,
        web_tx: &mut Sender<WebNotification>,
    ) -> Self {
        WebExampleFeature
    }

    fn set_receiver(&mut self, iot_feat: Arc<dyn IotFeature>) {}

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    async fn send(&mut self, notif: WebNotification) {}

    async fn recv(&mut self) -> IotNotification {
        IotNotification::None
    }

    fn create_router(&mut self) -> Router {
        Router::new().route("/api/health_check", get(WebExampleFeature::health_check))
    }

    async fn process_iot_notification(&mut self, notification: IotNotification) {}
}
