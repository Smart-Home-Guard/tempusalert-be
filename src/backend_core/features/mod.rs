use async_trait::async_trait;
use axum::Router;
use utoipa::openapi::PathItem;

use crate::notification::{IotNotification, WebNotification};

#[async_trait]
pub trait IotFeature<T, S: serde::Serialize> {
    fn name() -> String;
    async fn init(&mut self, mqtt_client: &mut rumqttc::Client);
    async fn process_iot_message(&mut self, message: T);
    async fn process_web_notification(&mut self, notification: WebNotification);
    async fn send_iot_command(&mut self, command: S);
}

#[async_trait]
pub trait WebFeature {
    fn create_router(&self) -> Router;
    fn create_swagger(&self) -> SwaggerMeta;
    async fn process_iot_notification(&mut self, notification: IotNotification);
}

pub struct SwaggerMeta {
    pub key: String,
    pub value: PathItem,
}



// Features
pub mod template_feature;