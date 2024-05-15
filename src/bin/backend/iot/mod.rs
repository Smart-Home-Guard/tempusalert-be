use mongodb::bson::Document;
use tempusalert_be::backend_core::features::IotFeature;

use crate::{
    clonable_wrapper::ClonableWrapper, config::IotConfig, globals::channels::{get_user_subscriber, UserEvent, UserEventKind}, types::IotFeatureDyn, AppResult
};

pub struct IotTask {
    pub config: IotConfig,
    features: Vec<ClonableWrapper<IotFeatureDyn>>,
}

impl IotTask {
    pub async fn create(
        config: IotConfig,
        features: Vec<ClonableWrapper<IotFeatureDyn>>,
    ) -> AppResult<Self> {
        Ok(Self { config, features })
    }

    pub async fn run(mut self) -> AppResult {
        let mut join_handles = vec![];
        for feat in &mut self.features {
            let feat_cloned = feat.clone();
            
            join_handles.push(tokio::spawn(async move {
                watch_users(feat_cloned).await;
            }));

            let mut feat_cloned = feat.clone();
            join_handles.push(tokio::spawn(async move {
                loop {
                    feat_cloned.process_next_mqtt_message().await;
                }
            }));
        }
        for handle in join_handles {
            handle.await.unwrap();
        }
        Ok(())
    }
}

async fn watch_users(mut feat: Box<dyn IotFeature + Send + Sync>) {
    let (feature_id, mqttc, mongoc) = {
        (feat.get_module_name(), feat.get_mqttc(), feat.get_mongoc())
    };
    let collection = mongoc.default_database().unwrap().collection("users");

    let mut user_cursor = collection.find(None, None).await.unwrap();
    while let Ok(true) = user_cursor.advance().await {
        let user_doc = user_cursor.deserialize_current();
        if let Some(cur_client_id) = user_doc.ok().and_then(|doc: Document| {
            doc.get("client_id")
                .and_then(|id| id.as_str())
                .map(|s| s.to_owned())
        }) {
            let mqtt_topic = format!("{}/{}-metrics", cur_client_id, feature_id);
            println!("Listen to existing user: {mqtt_topic}");

            if let Err(error) = mqttc
                .subscribe(mqtt_topic.clone(), rumqttc::QoS::AtLeastOnce)
                .await
            {
                eprintln!("Failed to subscribe to MQTT topic: {}", error);
            }
        }
    }

    // Watch on user insertion and user deletion
    let mut user_subscriber = get_user_subscriber().await;

    loop {
        match user_subscriber.recv().await {
            Ok(UserEvent {
                kind: UserEventKind::JOIN,
                client_id,
            }) => {
                let mqtt_topic = format!("{}/{}-metrics", client_id, feature_id);
                if let Err(e) = mqttc.subscribe(mqtt_topic.clone(), rumqttc::QoS::AtLeastOnce).await {
                    eprintln!(
                        "Error subscribing to a new user with client id {}: {}",
                        client_id, e
                    );
                } else {
                    println!("Listen to new user: {mqtt_topic}");
                }
            }

            Ok(UserEvent {
                kind: UserEventKind::CANCEL,
                client_id,
            }) => {
                let mqtt_topic = format!("{}/{}-metrics", client_id, feature_id);
                if let Err(e) = mqttc.unsubscribe(mqtt_topic.clone()).await {
                    eprintln!(
                        "Error unsubscribing from an old user with client id {}: {}",
                        client_id, e
                    );
                } else {
                    println!("Listen terminated for old user: {mqtt_topic}");
                }

            }

            Err(e) => {
                eprintln!("Error when listening on user channel {}", e);
            }
        }
    }
}
