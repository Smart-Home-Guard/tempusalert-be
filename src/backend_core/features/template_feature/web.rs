use axum::async_trait;
use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::result::Result;
use std::sync::Arc;

use crate::backend_core::features::{IotFeature, WebFeature};
use crate::errors::AppError;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct WebExampleFeature;

impl WebExampleFeature {
    pub async fn health_check() -> Result<Json<GenericResponse>, AppError> {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };
        Ok(Json(response_json))
    }
}

#[async_trait]
impl WebFeature for WebExampleFeature {
    fn create(mongoc: mongodb::Client) -> Self {
        WebExampleFeature
    }

    async fn init(&mut self, iot_feat: Arc<dyn IotFeature + Sync + Send>) {}

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    fn create_router(&mut self) -> Router {
        Router::new().route("/api/health_check", get(WebExampleFeature::health_check))
    }

    async fn run_loop(&mut self) {}
}
