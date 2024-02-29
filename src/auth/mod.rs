use hmac::{Hmac, Mac};
use jwt::{FromBase64, SignWithKey, VerifyWithKey};
use sha2::Sha256;
use serde::{Serialize};

pub fn sign_jwt(key: &str, claim: &impl Serialize) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    claim.sign_with_key(&key).ok()
}

pub fn decript_jwt<T: FromBase64>(key: &str, token_str: &str) -> Option<T> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(key.as_bytes()).ok()?;
    token_str.verify_with_key(&key).ok()
}

