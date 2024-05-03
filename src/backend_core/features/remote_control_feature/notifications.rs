use serde::{Deserialize, Serialize};

use super::models::{BuzzerCommand, LightCommand};

#[derive(Serialize, Deserialize)]
pub struct RemoteControlIotNotification {
    pub status_code: usize,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub enum RemoteControlWebNotification {
    BuzzerCommandNotification {
        device_id: usize,
        component_id: usize,
        command: BuzzerCommand,
        client_id: String,
    },
    LightCommandNotification {
        device_id: usize,
        component_id: usize,
        command: LightCommand,
        client_id: String,
    },
}
