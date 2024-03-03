use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::backend_core::features::{IotFeature, WebFeature};
use crate::json::Json;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebExampleFeature;

impl WebExampleFeature {
    async fn example(Json(_): Json<()>) -> impl IntoApiResponse {
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
    fn create(mongoc: mongodb::Client) -> Self {
        WebExampleFeature
    }

    async fn init(&mut self, iot_feat: Arc<Mutex<dyn IotFeature + Sync + Send>>) {}

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
