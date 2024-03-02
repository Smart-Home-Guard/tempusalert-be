use tempusalert_be::backend_core::features::IotFeature;

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
        let mut join_handles = vec![];
        for mut feat in self.features {
            join_handles.push(tokio::spawn(async move { feat.run_loop().await }));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }
}
