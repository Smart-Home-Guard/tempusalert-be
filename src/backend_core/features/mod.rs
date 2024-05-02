use std::{any::Any, sync::{Arc, Weak}};

use aide::axum::ApiRouter;
use axum::async_trait;

#[async_trait]
pub trait IotFeature {
    fn create(
        mqttc: rumqttc::AsyncClient,
        mqttc_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self>
    where
        Self: Sized;

    fn name() -> String
    where
        Self: Sized;

    fn get_module_name(&self) -> String;

    async fn process_next_mqtt_message(&mut self);

    async fn send_message_to_web(&mut self, message: String) -> String; 
    async fn response_message_from_web(&mut self, message: String) -> String;

    fn set_web_feature_instance<W: WebFeature + 'static>(&mut self, web_instance: Weak<W>)
    where
        Self: Sized; 

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;

    fn into_any(self: Arc<Self>) -> Arc<dyn Any>;
}

#[async_trait]
pub trait WebFeature {
    fn create(
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self>
    where
        Self: Sized;
    fn name() -> String
    where
        Self: Sized;
    fn get_module_name(&self) -> String;

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_instance: Weak<I>)
    where
        Self: Sized;
    
    fn create_router(&mut self) -> ApiRouter;
    
    async fn send_message_to_iot(&mut self, message: String) -> String;
    async fn response_message_from_iot(&mut self, message: String) -> String;

    fn into_any(self: Arc<Self>) -> Arc<dyn Any>;
}

// Features
pub mod devices_status_feature;
pub mod fire_alert_feature;
pub mod remote_control_feature;
pub mod template_feature;
