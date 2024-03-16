use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::{async_trait, http::StatusCode};
use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use super::notifications::{DeviceStatusIotNotification, DeviceStatusWebNotification};
use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;
use crate::json::Json;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebDeviceStatusFeature {
    mongoc: mongodb::Client,
    iot_tx: Sender<DeviceStatusWebNotification>,
    iot_rx: Receiver<DeviceStatusIotNotification>,
}

impl WebDeviceStatusFeature {
    async fn example() -> impl IntoApiResponse {
        let response_json = GenericResponse {
            status: "success".to_string(),
            message: "Example API".into(),
        };

        (StatusCode::OK, Json(response_json))
    }

    pub fn example_docs(op: TransformOperation) -> TransformOperation {
        op.description("Example api")
            .tag("Demo")
            .response::<200, Json<GenericResponse>>()
    }
}

#[async_trait]
impl WebFeature for WebDeviceStatusFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self> {
        let a = Some(WebDeviceStatusFeature {
            mongoc,
            iot_tx: non_primitive_cast(iot_tx)?,
            iot_rx: non_primitive_cast(iot_rx)?,
        });
        a
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

    fn create_router(&mut self) -> ApiRouter {
        ApiRouter::new().api_route(
            "/",
            get_with(
                WebDeviceStatusFeature::example,
                WebDeviceStatusFeature::example_docs,
            ),
        )
    }

    async fn process_next_iot_push_message(&mut self) {}
}
