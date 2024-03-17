use hmac::{Hmac, Mac};
use jwt::{FromBase64, SignWithKey, VerifyWithKey};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

pub fn sign_jwt(key: &str, claim: &impl Serialize) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    claim.sign_with_key(&key).ok()
}

pub fn decrypt_jwt<T: FromBase64>(key: &str, token_str: &str) -> Option<T> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    token_str.verify_with_key(&key).ok()
}

pub fn get_client_id_from_token(key: &str, token: String) -> Option<String> {
    let claim = decrypt_jwt::<IotClientClaim>(key, token.as_str())?;
    Some(claim.client_id)
}

pub fn get_email_from_token(
    key: &str,
    token: String,
) -> Option<String> {
    let claim = decrypt_jwt::<WebClientClaim>(key, token.as_str())?;
    Some(claim.email)
}

#[derive(Serialize, Deserialize)]
pub struct IotClientClaim {
    pub client_id: String,
    pub nonce: String,
}

#[derive(Serialize, Deserialize)]
pub struct WebClientClaim {
    pub email: String,
    pub nonce: String,
}
