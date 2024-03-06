use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::{async_trait, http::StatusCode};
use schemars::JsonSchema;
use serde::Serialize;
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
    fn create<ExampleWebNotification, ExampleIotNotification>(
        mongoc: mongodb::Client,
        iot_tx: Sender<ExampleWebNotification>,
        iot_rx: Receiver<ExampleIotNotification>,
    ) -> Self {
        WebExampleFeature
    }

    fn name() -> String where Self: Sized {
        "feature_example".into()
    }

    fn id(&self) -> String {
        "feature_example".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        ApiRouter::new().api_route(
            "/api/example",
            get_with(WebExampleFeature::example, WebExampleFeature::example_docs),
        )
    }

    async fn run_loop(&mut self) {}
}
