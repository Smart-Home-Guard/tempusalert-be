use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::{async_trait, http::StatusCode};
use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;
use crate::json::Json;
use super::notifications::{ExampleWebNotification, ExampleIotNotification};

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebExampleFeature {
    _mongoc: mongodb::Client,
    _iot_tx: Sender<ExampleWebNotification>,
    _iot_rx: Receiver<ExampleIotNotification>,
}

impl WebExampleFeature {
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
impl WebFeature for WebExampleFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self> {
        Some(WebExampleFeature {
            _mongoc: mongoc,
            _iot_tx: non_primitive_cast(iot_tx)?,
            _iot_rx: non_primitive_cast(iot_rx)?,
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

    fn create_router(&mut self) -> ApiRouter {
        ApiRouter::new().api_route(
            "/",
            get_with(WebExampleFeature::example, WebExampleFeature::example_docs),
        )
    }

    async fn run_loop(&mut self) {}
}
