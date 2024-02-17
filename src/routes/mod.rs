use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    configs::{ApiDoc, AppConfig},
    types,
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub async fn new(config: AppConfig) -> types::Result<Self> {
        Ok(Self {
            config: Arc::new(config),
        })
    }
}

mod server;

pub fn create_router_app(state: AppState) -> Router {
    let router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
    let router = server::add_routers(router);
    router.with_state(state)
}
