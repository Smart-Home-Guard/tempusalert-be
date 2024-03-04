use config::CONFIG;
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotTask;
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

#[tokio::main]
async fn main() -> AppResult {
    dotenv().ok();
    let config = CONFIG.clone();
    let (web_feats, iot_feats) = (vec![], vec![]);
    let _database_client_ = database_client::init(config.database.uri).await?;

    //TODO: add logic to get client id
    let mqtt_client_id = "hard code client id";
    let mqtt_client_config = ClientConfig {
        client_id: mqtt_client_id,
        broker_hostname: config.mqtt_client.hostname.as_str(),
        broker_port: config.mqtt_client.port,
        capacity: config.mqtt_client.capacity,
        keep_alive_sec: config.mqtt_client.keep_alive_sec,
    };
    let _mqtt_client_ = mqtt_client::init(mqtt_client_config);

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
