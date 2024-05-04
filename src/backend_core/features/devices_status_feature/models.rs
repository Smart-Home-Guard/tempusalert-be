use std::time::SystemTime;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::iot::mqtt_messages::ComponentType;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Device {
    pub id: u32,
    pub battery_logs: Vec<BatteryStatus>,
    pub error_logs: Vec<DeviceError>,
    pub components: Vec<Component>,
    pub owner_name: String, // username in document of collection User
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Component {
    pub id: u32,
    pub kind: ComponentType,
    pub logs: Vec<ComponentStatus>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ComponentStatus {
    Connect { timestamp: SystemTime },
    Disconnect { timestamp: SystemTime },
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct BatteryStatus {
    pub battery: u32,
    pub timestamp: SystemTime,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DeviceError {
    pub id: u32,
    pub component: u32,
    pub timestamp: SystemTime,
}
