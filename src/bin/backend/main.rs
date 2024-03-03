use std::sync::Arc;

use config::CONFIG;
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotTask;
use tempusalert_be::{
    backend_core::features::{
        template_feature::{IotExampleFeature, WebExampleFeature},
        IotFeature, WebFeature,
    },
    errors::AppError,
};
use tokio::sync::{mpsc, Mutex};
use web::WebTask;

mod config;
mod doc;
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
    let web_feats: Vec<Arc<Mutex<dyn WebFeature + Send + Sync>>> =
        vec![Arc::new(Mutex::new(WebExampleFeature))];
    let iot_feats: Vec<Arc<Mutex<dyn IotFeature + Send + Sync>>> =
        vec![Arc::new(Mutex::new(IotExampleFeature))];
    for index in 0..web_feats.len() {
        web_feats[index].lock().await.init(iot_feats[index].clone());
        iot_feats[index].lock().await.init(web_feats[index].clone());
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
