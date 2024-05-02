use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::{async_trait, http::StatusCode};
use schemars::JsonSchema;
use serde::Serialize;

use super::notifications::{ExampleIotNotification, ExampleWebNotification};
use crate::backend_core::features::template_feature::iot::IotExampleFeature;
use crate::backend_core::features::{IotFeature, WebFeature};
use crate::backend_core::utils::non_primitive_cast;
use crate::json::Json;

#[derive(Serialize, JsonSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[derive(Clone)]
pub struct WebExampleFeature {
    _mongoc: mongodb::Client,
    _iot_instance: Option<Box<IotExampleFeature>>,
    _jwt_key: String,
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
        jwt_key: String,
    ) -> Option<Self> {
        Some(WebExampleFeature {
            _mongoc: mongoc,
            _iot_instance: None,
            _jwt_key: jwt_key,
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

    fn set_iot_feature_instance<I: IotFeature + 'static>(&mut self, iot_feature: I) 
    where
        Self: Sized, 
    {
        self._iot_instance = Some(Box::new(non_primitive_cast(iot_feature).unwrap()));    
    }

    fn create_router(&mut self) -> ApiRouter {
        ApiRouter::new().api_route(
            "/",
            get_with(WebExampleFeature::example, WebExampleFeature::example_docs),
        )
    }

    async fn process_next_iot_push_message(&mut self) {}
}
