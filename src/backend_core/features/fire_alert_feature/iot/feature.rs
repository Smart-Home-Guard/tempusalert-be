use axum::async_trait;
use mongodb::{
    bson::{doc, to_bson, Document},
    Collection,
};
use rumqttc::{Event, Incoming, Publish};
use std::{sync::Arc, time::SystemTime};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use super::{
    super::notifications::{FireIotNotification, FireWebNotification},
    mqtt_messages::FireMQTTMessage,
};
use crate::{
    auth::get_email_from_client_token,
    backend_core::{
        features::{
            fire_alert_feature::models::{SensorDataType, SensorLogData},
            IotFeature,
        },
        utils::non_primitive_cast,
    }
};

pub struct IotFireFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    web_tx: Sender<FireIotNotification>,
    web_rx: Receiver<FireWebNotification>,
    jwt_key: String,
    fire_collection: Collection<Document>,
}

impl IotFireFeature {
    async fn process_sensor_data(
        &self,
        username: &str,
        sensor_data: Vec<SensorLogData>,
        sensor_type: SensorDataType,
    ) {
        let filter = doc! {"owner_name": username};
        let update_doc = self.generate_update_doc(&sensor_data, &sensor_type);

        let collection = self.fire_collection.clone();

        match collection
            .find_one_and_update(filter.clone(), update_doc.clone(), None)
            .await
        {
            Ok(result) => {
                if result.is_none() {
                    let insert_doc = doc! {
                        "$setOnInsert": {"owner_name": username},
                        "$set": update_doc
                    };

                    if let Err(err) = collection
                        .update_one(filter.clone(), insert_doc, None)
                        .await
                    {
                        eprintln!("Failed to upsert document: {}", err);
                    }
                }
            }
            Err(_) => {
                eprintln!("Failed to process sensor data");
            }
        }
    }

    fn generate_update_doc(
        &self,
        sensor_data: &[SensorLogData],
        sensor_type: &SensorDataType,
    ) -> Document {
        let field_name = match sensor_type {
            SensorDataType::Fire => "fire_logs",
            SensorDataType::Smoke => "smoke_logs",
            SensorDataType::CO => "co_logs",
            SensorDataType::Heat => "heat_logs",
            SensorDataType::FireButton => "fire_button_logs",
        };

        doc! {"$push": {field_name: { "$each": sensor_data.iter().map(|data| to_bson(data).unwrap()).collect::<Vec<_>>() }}}
    }
}

#[async_trait]
impl IotFeature for IotFireFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
        jwt_key: String,
    ) -> Option<Self> {
        Some(IotFireFeature {
            mqttc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc: mongoc.clone(),
            web_tx: non_primitive_cast(web_tx)?,
            web_rx: non_primitive_cast(web_rx)?,
            jwt_key,
            fire_collection: mongoc.default_database().unwrap().collection("Fire"),
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "fire-alert".into()
    }

    fn get_module_name(&self) -> String {
        "fire-alert".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    async fn process_next_mqtt_message(&mut self) {
        let mut mongoc = self.get_mongoc();
        let mut mqtt_event_loop = self.mqtt_event_loop.lock().await;
        if let Ok(Event::Incoming(Incoming::Publish(Publish { payload, .. }))) =
            mqtt_event_loop.poll().await
        {
            if let Ok(raw_json) = String::from_utf8(payload.to_vec()) {
                if let Some(message) = serde_json::from_str::<FireMQTTMessage>(&raw_json).ok() {
                    match message {
                        FireMQTTMessage::Safe {
                            token,
                            fire_data,
                            smoke,
                            co,
                            heat,
                            fire_button,
                        }
                        | FireMQTTMessage::Unsafe {
                            token,
                            fire_data,
                            smoke,
                            co,
                            heat,
                            fire_button,
                        } => {
                            if let Some(email) =
                                get_email_from_client_token(&self.jwt_key, token, &mut mongoc).await
                            {
                                let sensor_data = vec![
                                    (SensorDataType::Fire, fire_data),
                                    (SensorDataType::Smoke, smoke),
                                    (SensorDataType::CO, co),
                                    (SensorDataType::Heat, heat),
                                    (SensorDataType::FireButton, fire_button),
                                ];

                                for (sensor_type, data) in sensor_data {
                                    let sensor_logs = data
                                        .iter()
                                        .map(|sensor| SensorLogData {
                                            id: sensor.id,
                                            component: sensor.component.clone(),
                                            value: sensor.value,
                                            alert: sensor.alert,
                                            timestamp: SystemTime::now(),
                                        })
                                        .collect::<Vec<_>>();

                                    match sensor_type {
                                        SensorDataType::Fire
                                        | SensorDataType::Smoke
                                        | SensorDataType::CO
                                        | SensorDataType::Heat
                                        | SensorDataType::FireButton => {
                                            self.process_sensor_data(
                                                &email,
                                                sensor_logs,
                                                sensor_type,
                                            )
                                            .await;
                                        }
                                    }
                                }
                            } else {
                                eprintln!("Invalid token");
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to deserialize MQTT message");
                }
            } else {
                eprintln!("Failed to process MQTT message");
            }
        }
    }
    async fn process_next_web_push_message(&mut self) {}
}
