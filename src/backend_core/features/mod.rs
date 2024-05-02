use aide::axum::ApiRouter;
use axum::async_trait;

#[async_trait]
pub trait IotFeature: Clone {
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

    fn set_web_feature_instance<W: WebFeature + 'static>(&mut self, web_instance: W);

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;
}

#[async_trait]
pub trait WebFeature: Clone {
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

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_instance: I);
    
    fn create_router(&mut self) -> ApiRouter;
    
    async fn process_next_iot_push_message(&mut self);
}

// Features
pub mod devices_status_feature;
pub mod fire_alert_feature;
pub mod remote_control_feature;
pub mod template_feature;
