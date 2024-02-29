mod swagger;

use axum::Router;
use tempusalert_be::{
    core::features::{template_feature::WebFeatureExample, WebFeature}, notification::{IotNotification, WebNotification}
};
use tokio::sync::mpsc::{Receiver, Sender};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::swagger::ApiDoc;
use crate::{config::WebConfig, AppResult};

pub struct WebTask {
    pub config: WebConfig,
    tcp: tokio::net::TcpListener,
    pub iot_rx: Receiver<IotNotification>,
    pub iot_tx: Sender<WebNotification>,
}

impl WebTask {
    pub async fn create(
        mut config: WebConfig,
        iot_rx: Receiver<IotNotification>,
        iot_tx: Sender<WebNotification>,
    ) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        config.port = addr.port();
        Ok(Self {
            config,
            tcp,
            iot_rx,
            iot_tx,
        })
    }

    pub async fn run(self) -> AppResult {
        let router =
            Router::new().merge(SwaggerUi::new("/doc").url("/doc/openapi.json", ApiDoc::openapi()));
        let template_feature = WebFeatureExample::create_router();
        let router = router.nest("/", template_feature);
        axum::serve(self.tcp, router).await?;
        Ok(())
    }
}
