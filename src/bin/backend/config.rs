use std::{
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use config::Environment;
use once_cell::sync::Lazy;
use serde::Deserialize;

pub const ENV_PREFIX: &str = "APP";

pub static CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::read(get_env_source(ENV_PREFIX)).unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    pub addr: String,
    pub port: u16,
}

impl WebConfig {
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
    pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
        self.get_addr().parse()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct IotConfig {}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: WebConfig,
    pub iot: IotConfig,
}

impl AppConfig {
    pub fn read(env_src: Environment) -> Result<Self, config::ConfigError> {
        let config_dir = get_settings_dir();
        let profile = std::env::var("APP_PROFILE")
            .ok()
            .and_then(|env| Profile::from_str(&env).ok())
            .unwrap_or(Profile::Dev);
        let profile_filename = format!("{profile}.toml");
        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join(profile_filename)))
            .add_source(env_src)
            .build()?;
        config.try_deserialize()
    }
}

fn get_settings_dir() -> PathBuf {
    std::env::current_dir().unwrap().join("settings")
}

fn get_env_source(prefix: &str) -> config::Environment {
    config::Environment::with_prefix(prefix)
        .prefix_separator("__")
        .separator("__")
}

#[derive(
    Debug,
    strum::Display,
    strum::EnumString,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
)]
pub enum Profile {
    #[serde(rename = "test")]
    #[strum(serialize = "test")]
    Test,
    #[serde(rename = "dev")]
    #[strum(serialize = "dev")]
    Dev,
    #[serde(rename = "prod")]
    #[strum(serialize = "prod")]
    Prod,
}
