use std::sync::Arc;

use axum::Router;
use tempusalert_be::{
    backend_core::features::{WebFeature},
    message::{IotNotification, WebNotification},
};
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex};

use crate::{config::WebConfig, AppResult};

pub struct WebTask {
    pub config: WebConfig,
    tcp: tokio::net::TcpListener,
    pub web_rx: Arc<Mutex<Receiver<IotNotification>>>,
    pub web_tx: Sender<WebNotification>,
    features: Vec<Box<dyn WebFeature + Send>>,
    router: Router,
}

impl WebTask {
    pub async fn create(
        mut config: WebConfig,
        iot_rx: Arc<Mutex<Receiver<IotNotification>>>,
        iot_tx: Sender<WebNotification>,
        features: Vec<Box<dyn WebFeature + Send>>,
    ) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        config.port = addr.port();
        let router = Router::new();
        Ok(Self {
            config,
            tcp,
            web_rx: iot_rx,
            web_tx: iot_tx,
            features,
            router,
        })
    }

    pub async fn run(self) -> AppResult {
        let router = Router::new();
        axum::serve(self.tcp, router).await?;
        Ok(())
    }
}
