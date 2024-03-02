use std::sync::Arc;

use tempusalert_be::{backend_core::features::{IotFeature}, message::{IotNotification, WebNotification}};
use tokio::sync::{mpsc::{Receiver, Sender}, Mutex};

use crate::{config::IotConfig, AppResult};

pub struct IotTask {
    pub config: IotConfig,
    features: Vec<Box<dyn IotFeature + Send>>
}

impl IotTask {
    pub async fn create(
        config: IotConfig,
        features: Vec<Box<dyn IotFeature + Send>>,
    ) -> AppResult<Self> {
        Ok(Self {
            config,
            features,
        })
    }

    pub async fn run(self) -> AppResult {
        Ok(())
    }
}
