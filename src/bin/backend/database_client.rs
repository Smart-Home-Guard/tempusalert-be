use tempusalert_be::database_client::{self, MongocConfig};
use tokio::sync::OnceCell;

use crate::parse_env_var;

pub async fn init_database() -> mongodb::Client {
    let config = MongocConfig {
        auth_source: parse_env_var("MONGO_AUTH_SOURCE"),
        default_db: parse_env_var("MONGO_DEFAULT_DATABASE"),
        server_hostname: parse_env_var("MONGO_SERVER_HOSTNAME"),
        server_port: parse_env_var("MONGO_SERVER_PORT"),
        password: parse_env_var("MONGO_INITDB_ROOT_PASSWORD"),
        username: parse_env_var("MONGO_INITDB_ROOT_USERNAME"),
    };

    database_client::init(config)
        .await
        .ok()
        .expect("Failed to connect to database")
}

pub static MONGOC: OnceCell<mongodb::Client> = OnceCell::const_new();