mod iot;
mod notifications;
mod web;

pub use iot::IotDeviceStatusFeature as IotFeature;
pub use notifications::DeviceStatusIotNotification as IotNotification;
pub use notifications::DeviceStatusIotNotification as WebNotification;
pub use web::WebDeviceStatusFeature as WebFeature;
