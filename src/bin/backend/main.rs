use std::sync::Arc;

use config::CONFIG;
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotTask;
use tempusalert_be::{backend_core::features::{template_feature::{IotExampleFeature, WebExampleFeature}, WebFeature}, errors::AppError};
use tokio::sync::{mpsc, Mutex};
use web::WebTask;

mod config;
mod iot;
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
    let (iot_tx, web_rx) = mpsc::channel(64);
    let (web_tx, iot_rx) = mpsc::channel(64);
    let web_task = WebTask::create(config.server, Arc::new(Mutex::new(iot_rx)), iot_tx, vec![]).await?;
    let iot_task = IotTask::create(config.iot, Arc::new(Mutex::new(web_rx)), web_tx, vec![]).await?;

    join_all(vec![
        (true, web_task.run().boxed()),
        (true, iot_task.run().boxed()),
    ])
    .await
    .unwrap();

    Ok(())
}
