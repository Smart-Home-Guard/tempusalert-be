mod iot;
pub mod models;
mod notifications;
mod web;

pub use iot::IotRemoteControlFeature as IotFeature;
pub use notifications::RemoteControlIotNotification as IotNotification;
pub use notifications::RemoteControlWebNotification as WebNotification;
pub use web::WebRemoteControlFeature as WebFeature;
pub static MUST_ON: bool = true;
