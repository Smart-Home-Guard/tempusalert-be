use axum::Router;
use tempusalert_be::web_core::features::{
    template_feature::{FeatureExample, GenericResponse},
    Feature,
};
use utoipa::{Modify, OpenApi};
use utoipa_swagger_ui::SwaggerUi;

use crate::{configuration::AppConfig, AppResult};

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
        let template_feature = FeatureExample::create_swagger();
        openapi
            .paths
            .paths
            .insert(template_feature.key, template_feature.value);
    }
}

pub struct WebServer {
    config: AppConfig,
    tcp: tokio::net::TcpListener,
}

impl WebServer {
    pub async fn new(mut config: AppConfig) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.server.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        config.server.port = addr.port();
        Ok(Self { config, tcp })
    }

    pub async fn run(self) -> AppResult<()> {
        let router = Router::new()
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
        let template_feature = FeatureExample::create_router();
        let router = router.nest("/", template_feature);
        axum::serve(self.tcp, router).await?;
        Ok(())
    }
}
