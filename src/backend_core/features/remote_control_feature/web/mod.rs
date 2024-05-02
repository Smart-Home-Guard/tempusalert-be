use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::async_trait;
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex};

use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;

use super::{IotNotification, WebNotification};

mod routes;

pub struct WebRemoteControlFeature {
    mongoc: mongodb::Client,
    iot_tx: Sender<WebNotification>,
    iot_rx: Arc<Mutex<Receiver<IotNotification>>>,
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebRemoteControlFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebRemoteControlFeature {
            mongoc,
            iot_tx: non_primitive_cast(iot_tx)?,
            iot_rx: Arc::new(Mutex::new(non_primitive_cast(iot_rx)?)),
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "remote-control".into()
    }

    fn get_module_name(&self) -> String {
        "remote-control".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }

    async fn process_next_iot_push_message(&mut self) {}
}
