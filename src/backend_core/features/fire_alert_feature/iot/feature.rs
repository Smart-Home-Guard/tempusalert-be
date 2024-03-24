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
    },
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
    async fn persist_sensor_data(
        &self,
        owner_name: String,
        sensor_data: &[SensorLogData],
        sensor_type: &SensorDataType,
    ) -> Option<()> {
        let field_name = match sensor_type {
            SensorDataType::Fire => "fire_logs",
            SensorDataType::Smoke => "smoke_logs",
            SensorDataType::CO => "co_logs",
            SensorDataType::Heat => "heat_logs",
            SensorDataType::FireButton => "button_logs",
        };

        let fire_log_coll = self
            .mongoc
            .clone()
            .default_database()
            .unwrap()
            .collection("fire_alerts");
        let mut session = self.mongoc.clone().start_session(None).await.ok()?;
        session.start_transaction(None).await.ok()?;

        if let Ok(None) = fire_log_coll
            .find_one(
                doc! { "owner_name": owner_name.clone() },
                None,
                
            )
            .await
        {
            fire_log_coll.insert_one(doc! { "owner_name": owner_name.clone(), "fire_logs": [], "smoke_logs": [], "co_logs": [], "heat_logs": [], "button_logs": [] }, None).await.ok()?;
        }

        fire_log_coll.find_one_and_update(doc! { "owner_name": owner_name.clone() }, doc! { "$push": { field_name: { "$each": sensor_data.iter().map(|data| to_bson(data).unwrap()).collect::<Vec<_>>() }}}, None).await.ok()?;

        Some(())
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
                            button,
                        }
                        | FireMQTTMessage::Unsafe {
                            token,
                            fire_data,
                            smoke,
                            co,
                            heat,
                            button,
                        } => {
                            if let Some(email) =
                                get_email_from_client_token(&self.jwt_key, token, &mut mongoc).await
                            {
                                let sensor_data = vec![
                                    (SensorDataType::Fire, fire_data),
                                    (SensorDataType::Smoke, smoke),
                                    (SensorDataType::CO, co),
                                    (SensorDataType::Heat, heat),
                                    (SensorDataType::FireButton, button),
                                ];

                                for (sensor_type, data) in sensor_data {
                                    let sensor_logs = data
                                        .into_iter()
                                        .map(|sensor| SensorLogData {
                                            id: sensor.id,
                                            component: sensor.component,
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
                                            self.persist_sensor_data(
                                                email.clone(),
                                                &sensor_logs,
                                                &sensor_type,
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
