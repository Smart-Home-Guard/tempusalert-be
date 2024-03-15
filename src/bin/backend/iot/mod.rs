use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use mongodb::{bson::Document, change_stream::event::{ChangeStreamEvent, OperationType}, options::ChangeStreamOptions};
use tempusalert_be::backend_core::features::IotFeature;
use tokio::sync::Mutex;

use crate::{config::IotConfig, AppResult};

pub struct IotTask {
    pub config: IotConfig,
    features: Vec<Arc<Mutex<dyn IotFeature + Send + Sync>>>,
}

impl IotTask {
    pub async fn create(
        config: IotConfig,
        features: Vec<Arc<Mutex<dyn IotFeature + Send + Sync>>>,
    ) -> AppResult<Self> {
        Ok(Self { config, features })
    }

    pub async fn run(self) -> AppResult {
        let mut join_handles = vec![];
        for feat in self.features {
            let feat_cloned = feat.clone();
            join_handles.push(tokio::spawn(async move {
                watch_users(feat_cloned).await;
            }));

            let feat_cloned = feat.clone();
            join_handles.push(tokio::spawn(async move {
                loop {
                    let mut feat = feat_cloned.lock().await;
                    feat.process_next_mqtt_message().await;
                }
            }));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }
}

async fn watch_users(feat: Arc<Mutex<dyn IotFeature + Send + Sync>>) {
    let (feature_id, mqttc, mongoc) = {
            let mut feat = feat.lock().await;
            (feat.get_module_name(), feat.get_mqttc(), feat.get_mongoc())
    };
    let collection = mongoc.default_database().unwrap().collection("users");

    let mut user_cursor = collection.find(None, None).await.unwrap();
    while let Ok(_) = user_cursor.advance().await {
        let user_doc = user_cursor.deserialize_current();
        if let Some(cur_client_id) = user_doc.ok().and_then(|doc: Document| {
            doc.get("client_id")
            .and_then(|id| id.as_str())
            .map(|s| s.to_owned())
        }) {
            let mqtt_topic = format!("{}/{}-metrics", cur_client_id, feature_id);

            if let Err(error) = mqttc
                .subscribe(mqtt_topic.clone(), rumqttc::QoS::AtLeastOnce)
                .await {
                eprintln!("Failed to subscribe to MQTT topic: {}", error);
            }
            break;
        }
    }

    // Watch on user insertion and user deletion
    let change_stream_options = ChangeStreamOptions::builder()
        .full_document(Some(mongodb::options::FullDocumentType::UpdateLookup))
        .build();

    let mut change_stream = collection
        .watch(None, change_stream_options)
        .await
        .expect("Failed to create Change Stream cursor");

    while let Some(change_event) = change_stream.next().await {
        match change_event {
            Ok(ChangeStreamEvent {
                operation_type: OperationType::Insert,
                full_document,
                ..
            }) => {
                if let Some(new_client_id) = full_document.and_then(|doc: Document| {
                    doc.get("client_id")
                        .and_then(|id| id.as_str())
                        .map(|s| s.to_owned())
                }) {
                let mqtt_topic = format!("{}/{}-metrics", new_client_id, feature_id);

                if let Err(error) =
                    mqttc.subscribe(mqtt_topic, rumqttc::QoS::AtLeastOnce).await
                {
                    eprintln!("Failed to subscribe to MQTT topic: {}", error);
                }
            }
        }
            Ok(ChangeStreamEvent {
                operation_type: OperationType::Delete,
                full_document_before_change,
                ..
            }) => {
                if let Some(old_client_id) =
                    full_document_before_change.and_then(|doc: Document| {
                        doc.get("client_id")
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_owned())
                    })
                {
                    let mqtt_topic = format!("{}/{}-metrics", old_client_id, feature_id);

                    if let Err(error) = mqttc.unsubscribe(mqtt_topic).await {
                        eprintln!("Failed to unsubscribe from MQTT topic: {}", error);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => {
                eprintln!("Error processing change event: {}", error);
            }
        }
    }   
}
