pub mod iot;
pub mod models;
mod notifications;
pub mod web;

pub use iot::IotDeviceStatusFeature as IotFeature;
pub use notifications::DeviceStatusIotNotification as IotNotification;
pub use notifications::DeviceStatusWebNotification as WebNotification;
pub use web::WebDeviceStatusFeature as WebFeature;
pub static MUST_ON: bool = true;
