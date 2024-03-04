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
pub struct DatabaseConfig {
    pub uri: String,
}

// FIXME: bad implementation
// but if use the original ClientConfig, return referenced str out of block will be invalid
#[derive(Debug, Deserialize, Clone)]
pub struct MqttClientConfig {
    pub hostname: String,
    pub port: u16,
    pub capacity: usize,
    pub keep_alive_sec: u64,
}

impl MqttClientConfig {
    fn parse_env_var<T: std::str::FromStr>(var_name: &str, default: T) -> T {
        dotenv::var(var_name)
            .ok()
            .and_then(|val| val.parse().ok())
            .unwrap_or(default)
    }

    pub fn read_broker_config() -> Self {
        let mqtt_client_hostname =
            Self::parse_env_var("APP_MQTT_CLIENT_HOSTNAME", "0.0.0.0".to_string());
        let mqtt_client_port = Self::parse_env_var("APP_MQTT_CLIENT_PORT", 1883);
        let mqtt_client_capacity = Self::parse_env_var("APP_MQTT_CLIENT_CAPACITY", 100);
        let mqtt_client_keep_alive_sec = Self::parse_env_var("APP_MQTT_CLIENT_KEEP_ALIVE_SEC", 60);

        MqttClientConfig {
            hostname: mqtt_client_hostname,
            port: mqtt_client_port,
            capacity: mqtt_client_capacity,
            keep_alive_sec: mqtt_client_keep_alive_sec,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: WebConfig,
    pub iot: IotConfig,
    pub database: DatabaseConfig,
    pub mqtt_client: MqttClientConfig,
}

impl AppConfig {
    pub fn read(env_src: Environment) -> Result<Self, config::ConfigError> {
        let config_dir = get_settings_dir();
        let profile = std::env::var("APP_PROFILE")
            .ok()
            .and_then(|env| Profile::from_str(&env).ok())
            .unwrap_or(Profile::Dev);
        let profile_filename = format!("{profile}.toml");

        let database_url =
            dotenv::var("APP_DATABASE_URL").expect("DATABASE_URL not found in .env file");
        let database_config = DatabaseConfig { uri: database_url };

        let mqtt_client_config = MqttClientConfig::read_broker_config();

        let config = config::Config::builder()
            .add_source(config::File::from(config_dir.join(profile_filename)))
            .add_source(env_src)
            .build()?;

        let mut config: AppConfig = config.try_deserialize()?;
        config.database = database_config;
        config.mqtt_client = mqtt_client_config;

        Ok(config)
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
