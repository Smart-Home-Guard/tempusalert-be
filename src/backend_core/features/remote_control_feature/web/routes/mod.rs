use aide::axum::ApiRouter;

use super::WebRemoteControlFeature;


static mut WEB_INSTANCE: Option<WebRemoteControlFeature> = None;

mod control_light;
mod control_buzzer;

pub fn create_router(web_feature_instance: &mut WebRemoteControlFeature) -> ApiRouter {
    unsafe {
        WEB_INSTANCE = Some(web_feature_instance.clone());
    }

    ApiRouter::new().nest("/", control_buzzer::routes())
                    .nest("/", control_light::routes())
}
