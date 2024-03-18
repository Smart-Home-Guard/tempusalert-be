use mongodb::bson::oid::ObjectId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
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

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy, JsonSchema)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FireCollection {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner_name: String,
    pub fire_logs: Vec<SensorLogData>,
    pub smoke_logs: Vec<SensorLogData>,
    pub co_logs: Vec<SensorLogData>,
    pub heat_logs: Vec<SensorLogData>,
    pub fire_button_logs: Vec<SensorLogData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FireLog {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub owner_name: String,
    pub fire_logs: Vec<SensorLogData>,
    pub smoke_logs: Vec<SensorLogData>,
    pub co_logs: Vec<SensorLogData>,
    pub heat_logs: Vec<SensorLogData>,
    pub fire_button_logs: Vec<SensorLogData>,
}
