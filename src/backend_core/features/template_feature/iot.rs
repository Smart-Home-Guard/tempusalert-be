use async_trait::async_trait;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend_core::features::IotFeature;

pub struct IotExampleFeature;

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create<ExampleIotNotification, ExampleWebNotification>(mqttc: rumqttc::Client, mongoc: mongodb::Client, web_tx: Sender<ExampleIotNotification>, web_rx: Receiver<ExampleWebNotification>) -> Self {
        IotExampleFeature
    }

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    async fn run_loop(&mut self) {}
}
