use crate::{config::AppConfig, AppResult};

pub struct IotServer {
    config: AppConfig,
}

impl IotServer {
    pub async fn new(config: AppConfig) -> AppResult<Self> {
        Ok(Self { config })
    }

    pub async fn run(self) -> AppResult<()> {
        Ok(())
    }
}
