use hmac::{Hmac, Mac};
use jwt::{FromBase64, SignWithKey, VerifyWithKey};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::backend_core::models::User;

pub fn sign_jwt(key: &str, claim: &impl Serialize) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    claim.sign_with_key(&key).ok()
}

pub fn decrypt_jwt<T: FromBase64>(key: &str, token_str: &str) -> Option<T> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    token_str.verify_with_key(&key).ok()
}

pub fn get_client_id_from_client_token(key: &str, token: String) -> Option<String> {
    let claim = decrypt_jwt::<IotClientClaim>(key, token.as_str())?;
    Some(claim.client_id)
}

pub fn get_email_from_web_token(key: &str, token: String) -> Option<String> {
    let claim = decrypt_jwt::<WebClientClaim>(key, token.as_str())?;
    Some(claim.email)
}

pub async fn get_client_id_from_web_token(
    key: &str,
    token: String,
    mongoc: &mut mongodb::Client,
) -> Option<String> {
    let email = get_email_from_web_token(key, token)?;
    if let Ok(Some(user_doc)) = mongoc
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .find_one(doc! { "email": email }, None)
        .await
    {
        Some(user_doc.client_id)
    } else {
        None
    }
}

pub async fn get_email_from_client_token(
    key: &str,
    token: String,
    mongoc: &mut mongodb::Client,
) -> Option<String> {
    let client_id = get_client_id_from_client_token(key, token)?;
    if let Ok(Some(user_doc)) = mongoc
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .find_one(doc! { "client_id": client_id }, None)
        .await
    {
        Some(user_doc.email)
    } else {
        None
    }
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
