use tempusalert_be::database_client;
use tokio::sync::OnceCell;

use crate::parse_env_var;

pub async fn init_database() -> mongodb::Client {
    let database_url = parse_env_var("DATABASE_URL");
    database_client::init(database_url)
        .await
        .ok()
        .expect("Failed to connect to database")
}

pub static MONGOC: OnceCell<mongodb::Client> = OnceCell::const_new();
