mod devices;
mod rooms;
mod utils;

use aide::axum::ApiRouter;

pub fn room_routes() -> ApiRouter {
    ApiRouter::new()
        .nest("/", rooms::routes())
        .nest("/", devices::routes())
}
