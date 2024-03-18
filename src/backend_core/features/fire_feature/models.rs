use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize)]
pub struct SensorLogData {
    pub id: u32,
    pub component: String,
    pub value: u32,
    pub alert: FireStatus,
    pub timestamp: SystemTime,
}

pub enum SensorDataType {
    Fire,
    Smoke,
    CO,
    Heat,
    FireButton,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
pub enum FireStatus {
    SAFE,
    UNSAFE,
}

impl FireStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FireStatus::SAFE => "SAFE",
            FireStatus::UNSAFE => "UNSAFE",
        }
    }
}
