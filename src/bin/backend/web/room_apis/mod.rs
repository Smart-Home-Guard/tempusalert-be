mod add_device;
mod patch_device_of_rooms;
mod rooms;
mod utils;

use aide::axum::ApiRouter;

pub fn room_routes() -> ApiRouter {
    ApiRouter::new()
        .nest("/", rooms::routes())
        .nest("/devices", add_device::routes())
}
