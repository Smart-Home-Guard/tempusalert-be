use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RemoteControlResponse {
    success: bool,
    message: String,
}
