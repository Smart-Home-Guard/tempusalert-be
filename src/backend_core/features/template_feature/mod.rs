mod iot;
mod notifications;
mod web;

pub use iot::IotExampleFeature as IotFeature;
pub use notifications::ExampleIotNotification as IotNotification;
pub use notifications::ExampleWebNotification as WebNotification;
pub use web::WebExampleFeature as WebFeature;
pub static MUST_ON: bool = false;
