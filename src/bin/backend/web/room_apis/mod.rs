mod get_rooms;
mod patch_device_of_rooms;
mod patch_rooms;

use aide::axum::ApiRouter;

pub fn room_routes() -> ApiRouter {
    ApiRouter::new().nest("/", get_rooms::routes())
}
