use tempusalert_be::database_client::{self, MongocConfig, MongocReplicaMember};
use tokio::sync::OnceCell;

use crate::parse_env_var;

pub async fn init_database() -> mongodb::Client {
    let replica_members = parse_env_var::<String>("MONGO_REPLICA_ENDPOINTS").split(',').map(|e| {
       let mut split = e.split(':');
       let hostname = String::from(split.next().unwrap());
       let port = split.next().unwrap().parse::<u16>().unwrap();
       MongocReplicaMember { hostname, port }
    }).collect();
    let config = MongocConfig {
        replica_members,
        replica_set_name: parse_env_var("REPLICA_SET_NAME"),
        auth_source: parse_env_var("MONGO_AUTH_SOURCE"),
        default_db: parse_env_var("MONGO_DEFAULT_DATABASE"),
        password: parse_env_var("MONGO_INITDB_ROOT_PASSWORD"),
        username: parse_env_var("MONGO_INITDB_ROOT_USERNAME"),
    };

    database_client::init(config)
        .await
        .ok()
        .expect("Failed to connect to database")
}

pub static MONGOC: OnceCell<mongodb::Client> = OnceCell::const_new();
