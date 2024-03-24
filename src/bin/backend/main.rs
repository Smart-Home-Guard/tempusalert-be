use config::CONFIG;
use database_client::{init_database, MONGOC};
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotTask;
use rumqttc::{AsyncClient, EventLoop};
use tempusalert_be::{
    backend_core::features::{devices_status_feature,fire_alert_featuree, IotFeature, WebFeature},
    errors::AppError,
    mqtt_client::{self, ClientConfig},
};
use web::WebTask;

mod config;
mod database_client;
mod globals;
mod iot;
mod mail;
mod push_notification;

#[macro_use]
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

async fn init_mqtt_client(client_id: &str) -> (AsyncClient, EventLoop) {
    let mqtt_client_id = client_id;
    let mqtt_server_hostname: String = parse_env_var("MQTT_SERVER_HOSTNAME");
    let mqtt_server_port = parse_env_var("MQTT_SERVER_PORT");
    let mqtt_client_capacity = parse_env_var("MQTT_CLIENT_CAPACITY");
    let mqtt_client_keep_alive_sec = parse_env_var("MQTT_CLIENT_KEEP_ALIVE_SEC");
    let mqtt_client_config = ClientConfig {
        client_id: mqtt_client_id,
        broker_hostname: mqtt_server_hostname.as_str(),
        broker_port: mqtt_server_port,
        capacity: mqtt_client_capacity,
        keep_alive_sec: mqtt_client_keep_alive_sec,
    };
    mqtt_client::init(mqtt_client_config)
}

static mut TOGGABLE_FEATURES_NAMES: Vec<String> = vec![];

#[tokio::main]
async fn main() -> AppResult {
    dotenv().ok();
    let config = CONFIG.clone();
    let mongoc = MONGOC.get_or_init(init_database).await;

    let (web_feats, iot_feats, toggable_feat_names) = create_features!(
        mongoc.clone(),
        init_mqtt_client,
        fire_alert_feature,
        devices_status_feature
    );

    unsafe {
        TOGGABLE_FEATURES_NAMES = toggable_feat_names;
    }

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
