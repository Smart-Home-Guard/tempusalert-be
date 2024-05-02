use axum::{async_trait, http::StatusCode};
use rumqttc::{Event, Incoming, Publish};
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;
use crate::backend_core::{
    features::{
        remote_control_feature::web::WebRemoteControlFeature, IotFeature, WebFeature
    }, utils::non_primitive_cast,
};

#[derive(Clone)]
pub struct IotRemoteControlFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    web_instance: Option<Weak<WebRemoteControlFeature>>,
    jwt_key: String,
}

impl IotRemoteControlFeature {}

#[async_trait]
impl IotFeature for IotRemoteControlFeature {
    fn create(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self> {
        Some(IotRemoteControlFeature {
            mqttc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc: mongoc.clone(),
            web_instance: None,
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

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    async fn process_next_mqtt_message(&mut self) {
        let mut mongoc = self.get_mongoc();
        let mut mqtt_event_loop = self.mqtt_event_loop.lock().await;
        if let Ok(Event::Incoming(Incoming::Publish(Publish { payload, .. }))) =
            mqtt_event_loop.poll().await
        {
        }
    }

    fn set_web_feature_instance<W: WebFeature + 'static>(&mut self, web_instance: Weak<W>)
    where
        Self: Sized, 
    {
        self.web_instance = Some(non_primitive_cast(web_instance.clone()).unwrap());
    }

    async fn process_next_web_push_message(&mut self) { }
}
