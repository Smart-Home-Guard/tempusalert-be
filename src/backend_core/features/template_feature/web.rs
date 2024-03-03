use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use async_trait::async_trait;
use axum::http::StatusCode;
use schemars::JsonSchema;
use serde::{Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::backend_core::features::WebFeature;
use crate::json::Json;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebExampleFeature;

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
    fn create<ExampleWebNotification, ExampleIotNotification>(mongoc: mongodb::Client, iot_tx: Sender<ExampleWebNotification>, iot_rx: Receiver<ExampleIotNotification>) -> Self {
        WebExampleFeature
    }

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        ApiRouter::new().api_route(
            "/api/example",
            get_with(WebExampleFeature::example, WebExampleFeature::example_docs),
        )
    }

    async fn run_loop(&mut self) {}
}
