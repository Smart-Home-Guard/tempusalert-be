#[macro_use]
mod macros;

use utoipa::{Modify, OpenApi};
use tempusalert_be::web_core::features::{template_feature::{FeatureExample, GenericResponse}, Feature};

#[derive(OpenApi)]
#[openapi(
        info(
            version = "v0.1.0",
            title = "TEMPUSALERT API",
        ),
        components(
            schemas(
                GenericResponse,
            ),
            responses(
                GenericResponse
            )
        ),
        modifiers(&CustomPaths)
    )]
pub struct ApiDoc;

struct CustomPaths;

impl Modify for CustomPaths {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        insert_paths![openapi, FeatureExample];
    }
}