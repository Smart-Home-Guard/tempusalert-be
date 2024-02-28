use std::{
    net::{AddrParseError, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use config::{ConfigError, Environment};
use once_cell::sync::Lazy;
use serde::Deserialize;

pub const ENV_PREFIX: &str = "APP";

pub static CONFIG: Lazy<AppConfig> =
    Lazy::new(|| AppConfig::read(get_env_source(ENV_PREFIX)).unwrap());

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub addr: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub profile: Profile,
    pub server: ServerConfig,
    pub iot: ServerConfig,
}

impl ServerConfig {
    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
    pub fn get_socket_addr(&self) -> Result<SocketAddr, AddrParseError> {
        self.get_addr().parse()
    }
}

impl AppConfig {
    pub fn read(env_src: Environment) -> Result<Self, config::ConfigError> {
        let config_dir = get_settings_dir()?;
        let profile = std::env::var("APP_PROFILE")
            .map(|env| Profile::from_str(&env).map_err(|e| ConfigError::Message(e.to_string())))
            .unwrap_or_else(|_e| Ok(Profile::Dev))?;
        let profile_filename = format!("{profile}.toml");
        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join("base.toml")))
            .add_source(config::File::from(config_dir.join(profile_filename)))
            .add_source(env_src)
            .build()?;
        config.try_deserialize()
    }
}

pub fn get_settings_dir() -> Result<std::path::PathBuf, ConfigError> {
    Ok(get_project_root()
        .map_err(|e| ConfigError::Message(e.to_string()))?
        .join("settings"))
}

pub fn get_project_root() -> std::io::Result<PathBuf> {
    if let Some(root) = get_cargo_project_root()? {
        Ok(root)
    } else {
        Ok(std::env::current_dir()?)
    }
}

pub fn get_cargo_project_root() -> std::io::Result<Option<PathBuf>> {
    let current_path = std::env::current_dir()?;

    for ancestor in current_path.ancestors() {
        for dir in std::fs::read_dir(ancestor)? {
            let dir = dir?;
            if dir.file_name() == *"Cargo.lock" {
                return Ok(Some(ancestor.to_path_buf()));
            }
        }
    }
    Ok(None)
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
