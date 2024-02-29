use async_trait::async_trait;
use axum::Router;
use utoipa::openapi::PathItem;

#[async_trait]
pub trait IotFeature {
    type IotNotification;
    fn name() -> String;
    async fn init(&mut self, mqtt_client: &mut rumqttc::Client);
    async fn process_iot_message(&mut self, message: String);
    async fn process_push_notification(&mut self, message: String);
    async fn send_command(&mut self, command: String);
}

pub trait WebFeature {
    type WebNotification;
    fn create_router() -> Router;
    fn create_swagger() -> SwaggerMeta;
}

pub struct SwaggerMeta {
    pub key: String,
    pub value: PathItem,
}



// Features
pub mod template_feature;