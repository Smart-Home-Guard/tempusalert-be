use config::CONFIG;
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotTask;
use rumqttc::{AsyncClient, EventLoop};
use tempusalert_be::{
    database_client,
    errors::AppError,
    mqtt_client::{self, ClientConfig},
};
use web::WebTask;

mod config;
mod doc;
mod iot;
mod macros;
mod web;

pub type AppResult<T = ()> = std::result::Result<T, AppError>;

/// If a task is fail fast after encounter an error node goes down.
pub type IsFailFast = bool;
pub type Task = (IsFailFast, futures::future::BoxFuture<'static, AppResult>);

pub async fn join_all(tasks: Vec<Task>) -> AppResult {
    let (sender, mut receiver) = tokio::sync::mpsc::channel::<AppError>(1);
    for (is_fail_fast, task) in tasks {
        let sender = if is_fail_fast {
            Some(sender.clone())
        } else {
            None
        };
        tokio::spawn(async {
            if let Err(e) = task.await {
                if let Some(sender) = sender {
                    sender
                        .send(e)
                        .await
                        .unwrap_or_else(|_| unreachable!("This channel never closed."));
                } else {
                }
            }
        });
    }
    match receiver.recv().await {
        Some(err) => Err(err),
        None => unreachable!("This channel never closed."),
    }
}

fn parse_env_var<T: std::str::FromStr>(var_name: &str) -> T {
    dotenv::var(var_name)
        .ok()
        .and_then(|val| val.parse().ok())
        .expect(format!("{var_name} not found in environment variables").as_str())
}

async fn init_database() -> Option<mongodb::Client> {
    let database_url = parse_env_var("DATABASE_URL");
    database_client::init(database_url).await.ok()
}

async fn init_mqtt_client(client_id: &str) -> (AsyncClient, EventLoop) {
    let mqtt_client_id = client_id;
    let mqtt_client_hostname: String = parse_env_var("MQTT_CLIENT_HOSTNAME");
    let mqtt_client_port = parse_env_var("MQTT_CLIENT_PORT");
    let mqtt_client_capacity = parse_env_var("MQTT_CLIENT_CAPACITY");
    let mqtt_client_keep_alive_sec = parse_env_var("MQTT_CLIENT_KEEP_ALIVE_SEC");
    let mqtt_client_config = ClientConfig {
        client_id: mqtt_client_id,
        broker_hostname: mqtt_client_hostname.as_str(),
        broker_port: mqtt_client_port,
        capacity: mqtt_client_capacity,
        keep_alive_sec: mqtt_client_keep_alive_sec,
    };
    mqtt_client::init(mqtt_client_config)
}

#[tokio::main]
async fn main() -> AppResult {
    dotenv().ok();
    let config = CONFIG.clone();
    let (web_feats, iot_feats) = (vec![], vec![]);
    let mqtt_client_id = "hard code mqtt client id";

    let database_client = init_database().await.expect("Fail to initialize database.");
    let mqtt_client = init_mqtt_client(mqtt_client_id).await;
    let web_task = WebTask::create(config.server, web_feats).await?;
    let iot_task = IotTask::create(config.iot, iot_feats).await?;

    join_all(vec![
        (true, web_task.run().boxed()),
        (true, iot_task.run().boxed()),
    ])
    .await
    .unwrap();

    Ok(())
}
