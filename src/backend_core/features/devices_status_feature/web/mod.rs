use aide::axum::ApiRouter;
use axum::async_trait;
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
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebDeviceStatusFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebDeviceStatusFeature {
            mongoc,
            iot_tx: non_primitive_cast(iot_tx)?,
            iot_rx: non_primitive_cast(iot_rx)?,
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "devices-status".into()
    }

    fn get_module_name(&self) -> String {
        "devices-status".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }

    async fn process_next_iot_push_message(&mut self) {}
}
