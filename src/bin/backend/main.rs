use configuration::CONFIG;
use dotenv::dotenv;
use futures::FutureExt;
use iot::IotServer;
use tempusalert_be::web_core::error::AppError;
use web::WebServer;

mod configuration;
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
async fn main() {
    dotenv().ok();
    let config = CONFIG.clone();
    let web_server = WebServer::new(config).await.unwrap();
    let config = CONFIG.clone();
    let iot_server = IotServer::new(config).await.unwrap();

    join_all(vec![
        (true, web_server.run().boxed()),
        (true, iot_server.run().boxed()),
    ])
    .await
    .unwrap();
}
