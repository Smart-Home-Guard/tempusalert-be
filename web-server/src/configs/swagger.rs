use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

use crate::{dtos::*, features::Feature};

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

struct SecurityAddon;

struct CustomPaths;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}

impl Modify for CustomPaths {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let example = crate::features::template_feature::FeatureExample::new();
        example.add_swagger(openapi);

        let sample = crate::features::example_feature::FeatureSample::new();
        sample.add_swagger(openapi)
    }
}
