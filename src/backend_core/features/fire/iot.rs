use axum::async_trait;
use mongodb::bson::{doc, Document};
use mongodb::error::Error as MongoError;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::backend_core::{features::IotFeature, utils::non_primitive_cast};

use super::notifications::{FireIotNotification, FireWebNotification};

pub struct IotFireFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    web_tx: Sender<FireIotNotification>,
    web_rx: Receiver<FireWebNotification>,
}

impl IotFireFeature {}

#[async_trait]
impl IotFeature for IotFireFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Option<Self> {
        Some(IotFireFeature {
            mqttc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc,
            web_tx: non_primitive_cast(web_tx).unwrap(),
            web_rx: non_primitive_cast(web_rx).unwrap(),
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "feature_Fire".into()
    }

    fn get_module_name(&self) -> String {
        "feature_Fire".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    async fn run_loop(&mut self) {
        let database_name = match dotenv::var("MONGO_INITDB_DATABASE") {
            Ok(name) => name,
            Err(_) => {
                eprintln!("MONGO_INITDB_DATABASE not found in environment variables");
                return; // Abort loop if database name is not available
            }
        };

        let collection_name = "fireMessages";
        let collection =
            match create_or_get_collection(&self.mongoc, &database_name, collection_name).await {
                Ok(coll) => coll,
                Err(err) => {
                    eprintln!(
                        "Failed to create or get 'fireMessages' collection: {:?}",
                        err
                    );
                    return; // Abort loop if collection creation or retrieval failed
                }
            };

        let mqtt_event_loop = self.mqtt_event_loop.clone();
        loop {
            match mqtt_event_loop.lock().await.poll().await {
                Ok(event) => match event {
                    rumqttc::Event::Incoming(rumqttc::Incoming::Publish(publish)) => {
                        let payload_str = match std::str::from_utf8(&publish.payload) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!(
                                    "Error converting payload bytes to UTF-8 string: {:?}",
                                    e
                                );
                                continue; // Skip processing this message
                            }
                        };
                        let doc = Document::from(doc! { "message": payload_str });

                        if let Err(err) = collection.insert_one(doc, None).await {
                            eprintln!("Failed to insert message into MongoDB: {:?}", err);
                        }
                    }
                    _ => {}
                },
                Err(err) => {
                    eprintln!("Error occurred while polling MQTT event loop: {:?}", err);
                }
            }
        }
    }
}

async fn create_or_get_collection(
    mongoc: &mongodb::Client,
    database_name: &str,
    collection_name: &str,
) -> Result<mongodb::Collection<Document>, MongoError> {
    let database = mongoc.database(database_name);

    let collections = database.list_collection_names(None).await?;

    if !collections.contains(&collection_name.to_string()) {
        database.create_collection(collection_name, None).await?;
        println!("Created collection '{}'", collection_name);
    }

    Ok(database.collection(collection_name))
}
