use std::sync::Arc;

use aide::axum::ApiRouter;
use async_trait::async_trait;
use tokio::sync::Mutex;

#[async_trait]
pub trait IotFeature {
    fn create(mqttc: rumqttc::Client, mongoc: mongodb::Client) -> Self
    where
        Self: Sized;
    fn name(&mut self) -> String;
    async fn init(&mut self, web_feat: Arc<Mutex<dyn WebFeature + Sync + Send>>);
    async fn run_loop(&mut self);
}

#[async_trait]
pub trait WebFeature {
    fn create(mongoc: mongodb::Client) -> Self
    where
        Self: Sized;
    fn name(&mut self) -> String;
    async fn init(&mut self, iot_feat: Arc<Mutex<dyn IotFeature + Sync + Send>>);
    fn create_router(&mut self) -> ApiRouter;
    async fn run_loop(&mut self);
}

// Features
pub mod template_feature;
