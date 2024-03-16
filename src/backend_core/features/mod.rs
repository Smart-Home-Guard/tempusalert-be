use aide::axum::ApiRouter;
use axum::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait IotFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqttc_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Option<Self>
    where
        Self: Sized;

    fn name() -> String
    where
        Self: Sized;

    fn get_module_name(&self) -> String;

    async fn process_next_mqtt_message(&mut self);
    async fn process_next_web_push_message(&mut self);

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;
}

#[async_trait]
pub trait WebFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self>
    where
        Self: Sized;
    fn name() -> String
    where
        Self: Sized;
    fn get_module_name(&self) -> String;
    fn create_router(&mut self) -> ApiRouter;
    async fn process_next_iot_push_message(&mut self);
}

// Features
pub mod device_status_feature;
pub mod fire_feature;
pub mod template_feature;
