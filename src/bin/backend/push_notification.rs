use mongodb::{bson::doc, Cursor};
use once_cell::sync::Lazy;
use p256::{elliptic_curve::{rand_core::OsRng, PublicKey, SecretKey}, pkcs8::EncodePrivateKey, NistP256};
use tempusalert_be::backend_core::models::{PushCredential, PushKey};
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

static SECRET_KEY: Lazy<SecretKey<NistP256>> =
    Lazy::new(|| SecretKey::<NistP256>::random(&mut OsRng).to_pkcs8_pem(Default::default()).unwrap().to_string().parse::<SecretKey<NistP256>>().unwrap());

pub static PUBLIC_KEY: Lazy<PublicKey<NistP256>> =
    Lazy::new(|| SECRET_KEY.clone().public_key());

pub async fn push_notification(email: String, message: String, mongoc: &mut mongodb::Client) -> Option<()> {
    let mut cred_cursor: Cursor<PushCredential> = mongoc
        .default_database()
        .unwrap()
        .collection("push_credentials")
        .find(doc! { "email": email }, None)
        .await
        .ok()?;

    while let Ok(true) = cred_cursor.advance().await {
        let cred = cred_cursor.deserialize_current().ok()?;
        let endpoint = cred.endpoint;
        let PushKey { p256dh, auth } = cred.key;

        let subscription_info = SubscriptionInfo::new(endpoint, p256dh, auth);

        let sig_builder =
            VapidSignatureBuilder::from_pem(SECRET_KEY.to_bytes().as_slice(), &subscription_info)
                .ok()?
                .build()
                .ok()?;

        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        let content = message.as_bytes();
        builder.set_payload(ContentEncoding::Aes128Gcm, content);
        builder.set_vapid_signature(sig_builder);

        let client = IsahcWebPushClient::new().ok()?;

        client.send(builder.build().ok()?).await.ok()?;
    }

    Some(())
}
