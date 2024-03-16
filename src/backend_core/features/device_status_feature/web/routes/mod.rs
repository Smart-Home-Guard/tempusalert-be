use std::sync::Arc;

use aide::axum::ApiRouter;
use tokio::sync::Mutex;

use super::WebDeviceStatusFeature;

pub static mut MONGOC: Option<Arc<Mutex<mongodb::Client>>> = None;

mod get_all_devices;

pub fn create_router(web: &mut WebDeviceStatusFeature) -> ApiRouter {
    unsafe {
        MONGOC = Some(Arc::new(Mutex::new(web.mongoc.clone())));
    }

    ApiRouter::new()
        .nest_api_service("/", get_all_devices::routes())
}

