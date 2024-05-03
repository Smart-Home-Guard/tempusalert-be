use serde::Serialize;

use crate::backend_core::features::remote_control_feature::models::{BuzzerCommand, LightCommand};

#[derive(Serialize)]
pub struct LightRemoteControlCommand {
    pub device_id: usize,
    pub component_id: usize,
    pub command: LightCommand,
}

#[derive(Serialize)]
pub struct BuzzerRemoteControlCommand {
    pub device_id: usize,
    pub component_id: usize,
    pub command: BuzzerCommand,
}
