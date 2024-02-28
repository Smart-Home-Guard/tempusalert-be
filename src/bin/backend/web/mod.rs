mod swagger;

use axum::Router;
use tempusalert_be::web_core::features::{
    template_feature::FeatureExample,
    Feature,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::AppConfig, AppResult};
use self::swagger::ApiDoc;

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
            .merge(SwaggerUi::new("/doc").url("/doc/openapi.json", ApiDoc::openapi()));
        let template_feature = FeatureExample::create_router();
        let router = router.nest("/", template_feature);
        axum::serve(self.tcp, router).await?;
        Ok(())
    }
}
