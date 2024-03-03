use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use async_trait::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use schemars::JsonSchema;
use serde::Serialize;
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
    async fn health_check(Json(_): Json<()>) -> impl IntoApiResponse {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };

        (StatusCode::OK, Json(response_json))
    }

    pub fn health_check_docs(op: TransformOperation) -> TransformOperation {
        op.description("Example api")
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
            "/api/health_check",
            get_with(
                WebExampleFeature::health_check,
                WebExampleFeature::health_check_docs,
            ),
        )
    }

    async fn run_loop(&mut self) {}
}
