use tempusalert_be::notification::{IotNotification, WebNotification};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{config::IotConfig, AppResult};

pub struct IotTask {
    pub config: IotConfig,
    pub web_rx: Receiver<WebNotification>,
    pub web_tx: Sender<IotNotification>,
}

impl IotTask {
    pub async fn create(
        config: IotConfig,
        web_rx: Receiver<WebNotification>,
        web_tx: Sender<IotNotification>,
    ) -> AppResult<Self> {
        Ok(Self {
            config,
            web_rx,
            web_tx,
        })
    }

    pub async fn run(self) -> AppResult {
        Ok(())
    }
}
