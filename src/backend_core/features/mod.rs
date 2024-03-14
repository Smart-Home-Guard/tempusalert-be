use aide::axum::ApiRouter;
use axum::async_trait;
use futures::{stream::StreamExt, TryStreamExt};
use mongodb::{
    bson::Document,
    change_stream::event::{ChangeStreamEvent, OperationType},
    options::ChangeStreamOptions,
};
use tokio::sync::mpsc::{Receiver, Sender};

#[async_trait]
pub trait IotFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqttc_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Option<Self>
    where
        Self: Sized;

    fn name() -> String
    where
        Self: Sized;

    fn get_module_name(&self) -> String;

    async fn run_loop(&mut self);

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient;
    fn get_mongoc(&mut self) -> mongodb::Client;

    async fn watch_users(&mut self)
    where
        Self: Sized,
    {
        let mqttc = self.get_mqttc();
        let mongoc = self.get_mongoc();

        let database_name: String = dotenv::var("MONGO_INITDB_DATABASE")
            .ok()
            .and_then(|val| val.parse().ok())
            .expect("MONGO_INITDB_DATABASE not found in environment variables");
        let collection = mongoc.database(database_name.as_str()).collection("users");

        let mut user_stream = collection.find(None, None).await.unwrap();

        while let Ok(user_doc) = user_stream.try_next().await {
            if let Some(cur_client_id) = user_doc.and_then(|doc: Document| {
                doc.get("user_id")
                    .and_then(|id| id.as_str())
                    .map(|s| s.to_owned())
            }) {
                let feature_id = self.get_module_name();

                let mqtt_topic = format!("{}/{}-metrics", cur_client_id, feature_id);

                if let Err(error) = mqttc.subscribe(mqtt_topic, rumqttc::QoS::AtLeastOnce).await {
                    eprintln!("Failed to subscribe to MQTT topic: {}", error);
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
                        doc.get("user_id")
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_owned())
                    }) {
                        let feature_id = self.get_module_name();

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
                            doc.get("user_id")
                                .and_then(|id| id.as_str())
                                .map(|s| s.to_owned())
                        })
                    {
                        let feature_id = self.get_module_name();

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
}

#[async_trait]
pub trait WebFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self>
    where
        Self: Sized;
    fn name() -> String
    where
        Self: Sized;
    fn get_module_name(&self) -> String;
    fn create_router(&mut self) -> ApiRouter;
    async fn run_loop(&mut self);
}

// Features
pub mod fire;
pub mod template_feature;
