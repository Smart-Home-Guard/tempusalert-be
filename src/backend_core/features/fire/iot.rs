use axum::async_trait;
use mongodb::bson::{doc, Document};
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
        "fire".into()
    }

    fn get_module_name(&self) -> String {
        "fire".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    async fn run_loop(&mut self) {
        let mqtt_topic = format!(
            "974a4ab2-85cb-4eaa-b9dc-6414f32e8dfa/{}-metrics",
            self.get_module_name()
        );
        // FIXME: temporary use for testing, remove when watch_user block run_loop is solved
        if let Err(error) = self
            .mqttc
            .subscribe(mqtt_topic.clone(), rumqttc::QoS::AtLeastOnce)
            .await
        {
            eprintln!("Failed to subscribe to MQTT topic: {}", error);
        }

        let collection = self
            .mongoc
            .default_database()
            .unwrap()
            .collection("fireMessages");

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
