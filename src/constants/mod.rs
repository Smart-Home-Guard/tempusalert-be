use once_cell::sync::Lazy;

use crate::configs::env::get_env_source;

pub const ENV_PREFIX: &str = "APP";

pub static CONFIG: Lazy<crate::configs::AppConfig> =
    Lazy::new(|| crate::configs::AppConfig::read(get_env_source(ENV_PREFIX)).unwrap());
