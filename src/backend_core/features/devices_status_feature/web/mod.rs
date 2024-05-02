use std::any::Any;
use std::sync::{Arc, Weak};

use aide::axum::ApiRouter;
use axum::async_trait;
use schemars::JsonSchema;
use serde::Serialize;

use super::notifications::{DeviceStatusIotNotification, DeviceStatusWebNotification};
use crate::backend_core::features::devices_status_feature::iot::IotDeviceStatusFeature;
use crate::backend_core::features::{IotFeature, WebFeature};
use crate::backend_core::utils::non_primitive_cast;

mod routes;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Clone)]
pub struct WebDeviceStatusFeature {
    mongoc: mongodb::Client,
    _iot_instance: Option<Weak<IotDeviceStatusFeature>>,
    jwt_key: String,
}

#[async_trait]
impl WebFeature for WebDeviceStatusFeature {
    fn create(
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebDeviceStatusFeature {
            mongoc,
            _iot_instance: None,
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "devices-status".into()
    }

    fn get_module_name(&self) -> String {
        "devices-status".into()
    }

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_instance: Weak<I>)
    where
        Self: Sized, 
    {
        self._iot_instance = Some(non_primitive_cast(iot_instance.clone()).unwrap());
    }

    fn create_router(&mut self) -> ApiRouter {
        routes::create_router(self)
    }
    
    async fn send_message_to_iot(&mut self, message: String) -> String { String::from("") }
    async fn respond_message_from_iot(&mut self, message: String) -> String { String::from("") }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}
