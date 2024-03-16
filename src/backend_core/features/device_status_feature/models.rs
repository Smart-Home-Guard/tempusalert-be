use std::time::SystemTime;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Device {
    pub id: u32,
    pub battery_logs: Vec<BatteryStatus>,
    pub error_logs: Vec<DeviceError>,
    pub components: Vec<Component>,
}

#[derive(Serialize, Deserialize)]
pub struct Component {
    pub id: u32,
    pub logs: Vec<ComponentStatus>,
}

#[derive(Serialize, Deserialize)]
pub enum ComponentStatus {
    Connect {
        timestamp: SystemTime,
    },
    Disconnect {
        timestamp: SystemTime,
    },
}

#[derive(Serialize, Deserialize)]
pub struct BatteryStatus {
    pub battery: u32,
    pub timestamp: SystemTime,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceError {
    pub id: u32,
    pub component: u32,
    pub timestamp: SystemTime,
}
