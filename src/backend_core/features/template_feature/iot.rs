use axum::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend_core::features::IotFeature;

pub struct IotExampleFeature {
    mqttc: rumqttc::AsyncClient,
    mongoc: mongodb::Client,
}

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create<ExampleIotNotification, ExampleWebNotification>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<ExampleIotNotification>,
        web_rx: Receiver<ExampleWebNotification>,
    ) -> Self {
        IotExampleFeature { mqttc, mongoc }
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "feature_example".into()
    }

    fn id(&self) -> String {
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
