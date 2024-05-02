use std::sync::Arc;

use aide::axum::ApiRouter;
use tokio::sync::Mutex;

use super::WebRemoteControlFeature;

pub static mut MONGOC: Option<Arc<Mutex<mongodb::Client>>> = None;

mod control_light;
mod control_buzzer;

pub fn create_router(web_feature_instance: &mut WebRemoteControlFeature) -> ApiRouter {
    unsafe {
        MONGOC = Some(Arc::new(Mutex::new(web_feature_instance.mongoc.clone())));
    }

    ApiRouter::new().nest("/", control_buzzer::routes(web_feature_instance))
                    .nest("/", control_light::routes(web_feature_instance))
}
