use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
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
    LPG,
}

#[derive(Serialize_repr, Deserialize_repr, JsonSchema, Debug)]
#[repr(u8)]
#[cfg_attr(test, derive(std::cmp::PartialEq))]
pub enum FireStatus {
    SAFE = 0,
    UNSAFE = 1,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FireLog {
    pub owner_name: String,
    pub fire_logs: Vec<SensorLogData>,
    pub smoke_logs: Vec<SensorLogData>,
    pub co_logs: Vec<SensorLogData>,
    pub heat_logs: Vec<SensorLogData>,
    pub button_logs: Vec<SensorLogData>,
    pub lpg_logs: Vec<SensorLogData>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct Pagination {
    pub start_time: Option<i32>,
    pub end_time: Option<i32>,
    pub offset: Option<u64>,
    pub limit: Option<i64>,
}
