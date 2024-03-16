use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::{async_trait, http::StatusCode};
use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use super::notifications::{DeviceStatusIotNotification, DeviceStatusWebNotification};
use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;

mod routes;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebDeviceStatusFeature {
    mongoc: mongodb::Client,
    iot_tx: Sender<DeviceStatusWebNotification>,
    iot_rx: Receiver<DeviceStatusIotNotification>,
}

#[async_trait]
impl WebFeature for WebDeviceStatusFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self> {
        Some(WebDeviceStatusFeature {
            mongoc,
            iot_tx: non_primitive_cast(iot_tx)?,
            iot_rx: non_primitive_cast(iot_rx)?,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "device-status".into()
    }

    fn get_module_name(&self) -> String {
        "device-status".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self) 
    }

    async fn process_next_iot_push_message(&mut self) {}
}
