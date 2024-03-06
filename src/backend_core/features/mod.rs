use aide::axum::ApiRouter;
use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait IotFeature {
    fn create<I, W>(mqttc: rumqttc::AsyncClient, mqttc_event_loop: rumqttc::EventLoop, mongoc: mongodb::Client, web_tx: Sender<I>, web_rx: Receiver<W>) -> Self
    where
        Self: Sized;
    fn name() -> String where Self: Sized;
    fn id(&self) -> String;
    async fn run_loop(&mut self);
}

#[async_trait]
pub trait WebFeature {
    fn create<W, I>(mongoc: mongodb::Client, iot_tx: Sender<W>, iot_rx: Receiver<I>) -> Self
    where
        Self: Sized;
    fn name() -> String where Self: Sized;
    fn id(&self) -> String;
    fn create_router(&mut self) -> ApiRouter;
    async fn run_loop(&mut self);
}

// Features
pub mod template_feature;
