pub mod template_feature;

use axum::Router;
use utoipa::openapi::PathItem;

pub trait Feature {
    fn create_router() -> Router;
    fn create_swagger() -> SwaggerMeta;
}

pub struct SwaggerMeta {
    pub key: String,
    pub value: PathItem,
}
