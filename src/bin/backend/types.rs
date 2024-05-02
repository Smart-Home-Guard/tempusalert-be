use tempusalert_be::backend_core::features::{IotFeature, WebFeature};

pub type WebFeatureDyn = dyn WebFeature + Send + Sync + 'static;
pub type IotFeatureDyn = dyn IotFeature + Send + Sync + 'static;
