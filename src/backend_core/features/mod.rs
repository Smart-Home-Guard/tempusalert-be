use aide::axum::ApiRouter;
use async_trait::async_trait;
use futures::stream::StreamExt;
use mongodb::{bson::Document, options::ChangeStreamOptions};
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait IotFeature {
    fn create<I, W>(
        mqttc: rumqttc::AsyncClient,
        mqttc_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Self
    where
        Self: Sized;

    fn name() -> String
    where
        Self: Sized;

    fn id(&self) -> String;

    async fn run_loop(&mut self);

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;

    async fn watch_users(&mut self) {
        let mqttc = self.get_mqttc();
        let mongoc = self.get_mongoc();

        let database_name: String = dotenv::var("MONGO_INITDB_DATABASE")
            .ok()
            .and_then(|val| val.parse().ok())
            .expect("MONGO_INITDB_DATABASE not found in environment variables");
        let collection = mongoc.database(database_name.as_str()).collection("users");

        let change_stream_options = ChangeStreamOptions::builder()
            .full_document(Some(mongodb::options::FullDocumentType::UpdateLookup))
            .build();

        // Create a Change Stream cursor on the collection
        let mut change_stream = collection
            .watch(None, change_stream_options)
            .await
            .expect("Failed to create Change Stream cursor");

        // Process incoming change events
        while let Some(change_event) = change_stream.next().await {
            match change_event {
                Ok(_event) => {
                    if let Some(updated_user_id) = _event.full_document.and_then(|doc: Document| {
                        doc.get("user_id")
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_owned())
                    }) {
                        let feature_id = self.id();

                        let mqtt_topic = format!("{}/{}-metrics", updated_user_id, feature_id);

                        if let Err(error) =
                            mqttc.subscribe(mqtt_topic, rumqttc::QoS::AtLeastOnce).await
                        {
                            eprintln!("Failed to subscribe to MQTT topic: {}", error);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("Error processing change event: {}", error);
                }
            }
        }
    }
}

#[async_trait]
pub trait WebFeature {
    fn create<W, I>(mongoc: mongodb::Client, iot_tx: Sender<W>, iot_rx: Receiver<I>) -> Self
    where
        Self: Sized;
    fn name() -> String
    where
        Self: Sized;
    fn id(&self) -> String;
    fn create_router(&mut self) -> ApiRouter;
    async fn run_loop(&mut self);
}

// Features
pub mod template_feature;
