use utoipa::{
    openapi::security::{Http, HttpAuthScheme, SecurityScheme},
    Modify, OpenApi,
};

use crate::dtos::*;

#[derive(OpenApi)]
#[openapi(
    info(
        version = "v0.1.0",
        title = "TEMPUSALERT API",
    ),
    paths(
        // server api 
        crate::handlers::server::health_check,

    ),
    components(
        schemas(
            GenericResponse,
        )
    ),
    tags(
        (name = "crate::handlers::server", description = "server endpoints."),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        )
    }
}
