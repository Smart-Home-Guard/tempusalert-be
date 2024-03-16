use ring::digest::SHA512_OUTPUT_LEN;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub client_id: String,
    #[serde(with = "serde_bytes")]
    pub client_secret: [u8; 10],
    pub email: String,
    #[serde(with = "serde_bytes")]
    pub hashed_password: [u8; SHA512_OUTPUT_LEN],
    #[serde(with = "serde_bytes")]
    pub salt: [u8; SHA512_OUTPUT_LEN],
}