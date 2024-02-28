pub mod template_feature;

use axum::Router;
use utoipa::openapi::PathItem;

pub trait Feature<S: Clone + Send + Sync + 'static = ()> {
    fn create_router() -> Router<S>;
    fn create_swagger() -> SwaggerMeta;
}

pub struct SwaggerMeta {
    key: String,
    value: PathItem,
}
