use std::sync::Arc;

use aide::axum::ApiRouter;
use tokio::sync::Mutex;

use super::WebDeviceStatusFeature;

pub static mut MONGOC: Option<Arc<Mutex<mongodb::Client>>> = None;

mod get_all_devices;
mod get_device_by_id;

pub fn create_router(web: &mut WebDeviceStatusFeature) -> ApiRouter {
    unsafe {
        MONGOC = Some(Arc::new(Mutex::new(web.mongoc.clone())));
    }

    ApiRouter::new()
        .nest("/", get_all_devices::routes())
        .nest("/", get_device_by_id::routes())
}

