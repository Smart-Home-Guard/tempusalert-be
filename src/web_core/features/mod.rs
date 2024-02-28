use crate::web_core::routes::AppState;

pub mod template_feature;
pub mod example_feature;

pub trait Feature {
    fn new() -> Self;
    fn add_routers(router: axum::Router<AppState>) -> axum::Router<AppState>;
    fn add_swagger(&self, openapi: &mut utoipa::openapi::OpenApi);
}
