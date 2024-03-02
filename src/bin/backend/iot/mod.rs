use std::sync::Arc;

use tempusalert_be::backend_core::features::IotFeature;
use tokio::sync::Mutex;

use crate::{config::IotConfig, AppResult};

pub struct IotTask {
    pub config: IotConfig,
    features: Vec<Arc<Mutex<dyn IotFeature + Send + Sync>>>,
}

impl IotTask {
    pub async fn create(
        config: IotConfig,
        features: Vec<Arc<Mutex<dyn IotFeature + Send + Sync>>>,
    ) -> AppResult<Self> {
        Ok(Self {
            config,
            features,
        })
    }

    pub async fn run(self) -> AppResult {
        let mut join_handles = vec![];
        for feat in self.features {
            join_handles.push(tokio::spawn(async move { feat.lock().await.run_loop().await }));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }
}
