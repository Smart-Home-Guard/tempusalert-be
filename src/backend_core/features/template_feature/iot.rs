use std::sync::{Arc, Weak};

use axum::async_trait;
use tokio::sync::Mutex;

use crate::backend_core::{features::{IotFeature, WebFeature}, utils::non_primitive_cast};

use super::{notifications::{ExampleIotNotification, ExampleWebNotification}, web::WebExampleFeature};

#[derive(Clone)]
pub struct IotExampleFeature {
    mqttc: rumqttc::AsyncClient,
    _mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    _web_instance: Option<Weak<WebExampleFeature>>,
    _jwt_key: String,
}

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self> {
        Some(IotExampleFeature {
            mqttc,
            _mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc,
            _web_instance: None,
            _jwt_key: jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "feature_example".into()
    }

    fn get_module_name(&self) -> String {
        "feature_example".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }
    
    fn set_web_feature_instance<W: WebFeature + 'static>(&mut self, web_instance: Weak<W>) 
    where
        Self: Sized,
    {
        self._web_instance = Some(non_primitive_cast(web_instance.clone()).unwrap());
    }

    async fn process_next_mqtt_message(&mut self) {}
    async fn process_next_web_push_message(&mut self) {}
}
