use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend_core::features::IotFeature;

pub struct IotExampleFeature;

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create<ExampleIotNotification, ExampleWebNotification>(mqttc: rumqttc::AsyncClient, mqtt_event_loop: rumqttc::EventLoop, mongoc: mongodb::Client, web_tx: Sender<ExampleIotNotification>, web_rx: Receiver<ExampleWebNotification>) -> Self {
        IotExampleFeature
    }

    fn name() -> String where Self: Sized {
        "feature_example".into()
    }

    fn id(&self) -> String {
        "feature_example".into()
    }

    async fn run_loop(&mut self) {}
}
