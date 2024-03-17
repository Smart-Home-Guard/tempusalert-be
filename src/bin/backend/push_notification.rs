use std::fs::File;

use mongodb::{bson::doc, Cursor};
use once_cell::sync::Lazy;
use web_push::{
    ContentEncoding, IsahcWebPushClient, SubscriptionInfo, VapidSignatureBuilder, WebPushClient,
    WebPushMessageBuilder,
};

use crate::{
    models::{PushCredential, PushKey},
    parse_env_var,
};

static PEM_FILE: Lazy<File> =
    Lazy::new(|| File::open(parse_env_var::<String>("PEM_FILE")).unwrap());

pub async fn push_notification(email: String, mongoc: &mut mongodb::Client) -> Option<()> {
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
            VapidSignatureBuilder::from_pem(PEM_FILE.try_clone().ok()?, &subscription_info)
                .ok()?
                .build()
                .ok()?;

        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        let content = "Encrypted payload to be sent in the notification".as_bytes();
        builder.set_payload(ContentEncoding::Aes128Gcm, content);
        builder.set_vapid_signature(sig_builder);

        let client = IsahcWebPushClient::new().ok()?;

        client.send(builder.build().ok()?).await.ok()?;
    }

    Some(())
}
