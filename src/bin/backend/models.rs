use ring::digest::SHA512_OUTPUT_LEN;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub client_id: String,
    pub client_secret: String,
    pub email: String,
    #[serde(with = "serde_bytes")]
    pub hashed_password: [u8; SHA512_OUTPUT_LEN],
    #[serde(with = "serde_bytes")]
    pub salt: [u8; SHA512_OUTPUT_LEN],
    pub enabled_features: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PushCredential {
    pub endpoint: String,
    pub key: PushKey,
    pub email: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct PushKey {
    pub p256dh: String,
    pub auth: String,
}
