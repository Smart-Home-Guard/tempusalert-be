use aide::axum::ApiRouter;
use axum::async_trait;
use schemars::JsonSchema;
use serde::Serialize;

use super::notifications::{FireIotNotification, FireWebNotification};
use crate::backend_core::features::fire_alert_feature::iot::IotFireFeature;
use crate::backend_core::features::{IotFeature, WebFeature};
use crate::backend_core::utils::non_primitive_cast;

mod routes;

#[derive(Serialize, JsonSchema)]
pub struct FireResponse {
    pub status: String,
    pub message: String,
}

struct FireAppState {
    mongoc: mongodb::Client,
}

#[derive(Clone)]
pub struct WebFireFeature {
    mongoc: mongodb::Client,
    _iot_instance: Option<Box<IotFireFeature>>,
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebFireFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebFireFeature {
            mongoc,
            _iot_instance: None,
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "fire-alert".into()
    }

    fn get_module_name(&self) -> String {
        "fire-alert".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_instance: I) {
        self._iot_instance = Some(Box::new(non_primitive_cast(iot_instance).unwrap()));
    }

    async fn process_next_iot_push_message(&mut self) {}
}
