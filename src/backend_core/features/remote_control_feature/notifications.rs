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
        client_id: String,
    },
    LightCommandNotification {
        device_id: usize,
        component_id: usize,
        command: LightCommand,
        client_id: String,
    },
}
