use once_cell::sync::Lazy;

use crate::web_core::configs::env::get_env_source;

pub const ENV_PREFIX: &str = "APP";

pub static CONFIG: Lazy<crate::web_core::configs::AppConfig> =
    Lazy::new(|| crate::web_core::configs::AppConfig::read(get_env_source(ENV_PREFIX)).unwrap());
