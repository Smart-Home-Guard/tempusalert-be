mod add_device;
mod get_rooms;
mod patch_device_of_rooms;
mod patch_rooms;
mod utils;

use aide::axum::ApiRouter;

pub fn room_routes() -> ApiRouter {
    ApiRouter::new()
        .nest("/", get_rooms::routes())
        .nest("/", add_device::routes())
}
