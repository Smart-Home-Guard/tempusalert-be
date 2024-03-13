use std::sync::Arc;

use axum::async_trait;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::backend_core::{features::IotFeature, utils::non_primitive_cast};

use super::notifications::{ExampleIotNotification, ExampleWebNotification};

pub struct IotExampleFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    web_tx: Sender<ExampleIotNotification>,
    web_rx: Receiver<ExampleWebNotification>,
}

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Option<Self> {
        Some(IotExampleFeature {
            mqttc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc,
            web_tx: non_primitive_cast(web_tx)?,
            web_rx: non_primitive_cast(web_rx)?,
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

    async fn run_loop(&mut self) {}
}
