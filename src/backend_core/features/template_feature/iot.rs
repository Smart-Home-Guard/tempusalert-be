use std::sync::Arc;

use async_trait::async_trait;

use crate::backend_core::features::{IotFeature, WebFeature};

pub struct IotExampleFeature;

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create(mqttc: rumqttc::Client, mongoc: mongodb::Client) -> Self {
        IotExampleFeature
    }

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    async fn init(&mut self, web_feat: Arc<dyn WebFeature + Sync + Send>) {}

    async fn run_loop(&mut self) {}
}
