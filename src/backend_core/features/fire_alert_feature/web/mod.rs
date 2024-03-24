use aide::axum::ApiRouter;
use axum::async_trait;
use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use super::notifications::{FireIotNotification, FireWebNotification};
use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;

mod routes;

#[derive(Serialize, JsonSchema)]
pub struct FireResponse {
    pub status: String,
    pub message: String,
}

struct FireAppState {
    mongoc: mongodb::Client,
}

pub struct WebFireFeature {
    mongoc: mongodb::Client,
    iot_tx: Sender<FireWebNotification>,
    iot_rx: Receiver<FireIotNotification>,
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebFireFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebFireFeature {
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
        "fire-alert".into()
    }

    fn get_module_name(&self) -> String {
        "fire-alert".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }

    async fn process_next_iot_push_message(&mut self) {}
}
