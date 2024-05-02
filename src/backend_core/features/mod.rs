use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::async_trait;

use super::utils::non_primitive_cast;

#[async_trait]
pub trait IotFeature {
    fn create<I: 'static, W: 'static>(
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
    async fn process_next_web_push_message(&mut self);

    fn set_web_feature_instance<W: WebFeature + 'static + Sized>(&mut self, web_instance: Arc<W>)
    where
        Self: Sized; 

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;
}

#[async_trait]
pub trait WebFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self>
    where
        Self: Sized;
    fn name() -> String
    where
        Self: Sized;
    fn get_module_name(&self) -> String;

    fn set_iot_feature_instance<I: IotFeature + 'static + Sized>(&mut self, iot_instance: Arc<I>)
    where
        Self: Sized;
    
    fn create_router(&mut self) -> ApiRouter;
    
    async fn process_next_iot_push_message(&mut self);
}

// Features
pub mod devices_status_feature;
pub mod fire_alert_feature;
pub mod remote_control_feature;
pub mod template_feature;
