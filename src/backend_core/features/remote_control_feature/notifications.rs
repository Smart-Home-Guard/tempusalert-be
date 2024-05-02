use axum::http::StatusCode;

use super::models::{BuzzerCommand, LightCommand};

pub struct RemoteControlIotNotification {
    pub status_code: StatusCode,
    pub message: String,
}

pub enum RemoteControlWebNotification {
    BuzzerCommandNotification {
        device_id: usize,
        component_id: usize,
        command: BuzzerCommand,
    },
    LightCommandNotification {
        device_id: usize,
        component_id: usize,
        command: LightCommand,
    },
}
