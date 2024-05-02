use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum BuzzerCommand {
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum LightCommand {
    #[serde(rename = "toggle")]
    Toggle,
    #[serde(rename = "on")]
    On,
    #[serde(rename = "off")]
    Off,
}
