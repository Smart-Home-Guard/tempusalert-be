use std::sync::Arc;

use aide::axum::ApiRouter;
use axum::async_trait;

use crate::backend_core::{features::{IotFeature, WebFeature}, utils::non_primitive_cast};

use super::iot::IotRemoteControlFeature;

mod routes;

#[derive(Clone)]
pub struct WebRemoteControlFeature {
    mongoc: mongodb::Client,
    iot_instance: Option<Arc<IotRemoteControlFeature>>,
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebRemoteControlFeature {
    fn create(
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebRemoteControlFeature {
            mongoc,
            iot_instance: None,
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

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_instance: Arc<I>) 
    where
        Self: Sized, 
    {
        self.iot_instance = Some(non_primitive_cast(iot_instance.clone()).unwrap());
    }

    async fn process_next_iot_push_message(&mut self) {}
}
