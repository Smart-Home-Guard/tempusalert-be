use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SensorLogData {
    pub id: u32,
    pub component: u32,
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

#[derive(Deserialize, Serialize, JsonSchema)]
#[cfg_attr(test, derive(std::cmp::PartialEq, Debug))]
pub enum FireStatus {
    SAFE,
    UNSAFE,
}

#[derive(Serialize, Deserialize)]
pub struct FireLog {
    pub owner_name: String,
    pub fire_logs: Vec<SensorLogData>,
    pub smoke_logs: Vec<SensorLogData>,
    pub co_logs: Vec<SensorLogData>,
    pub heat_logs: Vec<SensorLogData>,
    pub fire_button_logs: Vec<SensorLogData>,
}
