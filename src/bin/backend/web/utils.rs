use std::num::NonZeroU32;

use once_cell::sync::Lazy;
use ring::{digest, pbkdf2, rand::{self, SecureRandom}};

const N_ITER: Lazy<NonZeroU32> = Lazy::new(|| NonZeroU32::new(100_000).unwrap());

pub fn hash_password(password: String) -> Option<([u8; digest::SHA512_OUTPUT_LEN], [u8; digest::SHA512_OUTPUT_LEN])> {
    const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
    let rng = rand::SystemRandom::new();

    let mut salt = [0u8; CREDENTIAL_LEN];
    rng.fill(&mut salt).ok()?;

    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        N_ITER.to_owned(),
        &salt,
        password.as_bytes(),
        &mut pbkdf2_hash,
    );

    Some((pbkdf2_hash, salt))
}

pub fn verify_hashed_password(sent_password: String, hashed_password: [u8; digest::SHA512_OUTPUT_LEN], salt: [u8; digest::SHA512_OUTPUT_LEN]) -> bool {
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA512,
        N_ITER.to_owned(),
        &salt,
        sent_password.as_bytes(),
        &hashed_password,
    ).is_ok()
}