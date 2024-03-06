mod iot;
mod web;
mod notifications;

pub use iot::IotExampleFeature as IotFeature;
pub use web::WebExampleFeature as WebFeature;
pub use notifications::ExampleIotNotification as IotNotification;
pub use notifications::ExampleWebNotification as WebNotification;