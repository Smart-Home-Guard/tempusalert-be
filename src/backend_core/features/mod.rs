use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::message::{IotCommand, IotMessage, IotNotification, WebNotification};

#[async_trait]
pub trait IotFeature {
    fn create(
        web_rx: Arc<Mutex<Receiver<WebNotification>>>,
        web_tx: &mut Sender<IotNotification>,
    ) -> Self
    where
        Self: Sized;
    fn set_receiver(&mut self, web_feat: Arc<dyn WebFeature>);
    async fn send(&mut self, notif: IotNotification);
    async fn recv(&mut self) -> WebNotification;
    fn name(&mut self) -> String;
    async fn init(&mut self, mqtt_client: &mut rumqttc::Client);
    async fn process_iot_message(&mut self, message: IotMessage);
    async fn process_web_notification(&mut self, notification: WebNotification);
    async fn send_iot_command(&mut self, command: IotCommand);
}

#[async_trait]
pub trait WebFeature {
    fn create(
        web_rx: Arc<Mutex<Receiver<IotNotification>>>,
        web_tx: &mut Sender<WebNotification>,
    ) -> Self
    where
        Self: Sized;
    fn set_receiver(&mut self, web_feat: Arc<dyn IotFeature>);
    fn name(&mut self) -> String;
    async fn send(&mut self, notif: WebNotification);
    async fn recv(&mut self) -> IotNotification;
    fn create_router(&mut self) -> Router;
    async fn process_iot_notification(&mut self, notification: IotNotification);
}

// Features
pub mod template_feature;
