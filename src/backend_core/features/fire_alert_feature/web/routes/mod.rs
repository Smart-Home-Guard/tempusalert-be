use std::sync::Arc;

use aide::axum::ApiRouter;
use tokio::sync::Mutex;

use super::WebFireFeature;

pub static mut MONGOC: Option<Arc<Mutex<mongodb::Client>>> = None;

mod get_logs_of_user;
mod get_button_logs_of_user;
mod get_co_logs_of_user;
mod get_fire_logs_of_user;
mod get_gas_logs_of_user;
mod get_heat_logs_of_user;
mod get_smoke_logs_of_user;
mod get_buzzer_logs;
mod get_light_logs;

pub fn create_router(web: &mut WebFireFeature) -> ApiRouter {
    unsafe {
        MONGOC = Some(Arc::new(Mutex::new(web.mongoc.clone())));
    }

    ApiRouter::new().nest("/", get_logs_of_user::routes())
                    .nest("/", get_button_logs_of_user::routes())
                    .nest("/", get_co_logs_of_user::routes())
                    .nest("/", get_gas_logs_of_user::routes())
                    .nest("/", get_heat_logs_of_user::routes())
                    .nest("/", get_smoke_logs_of_user::routes())
                    .nest("/", get_fire_logs_of_user::routes())
                    .nest("/", get_buzzer_logs::routes())
                    .nest("/", get_light_logs::routes())
}
