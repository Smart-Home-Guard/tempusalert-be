use std::{
    any::Any,
    sync::{Arc, Weak},
    time::SystemTime,
};

use axum::async_trait;
use mongodb::{
    bson::{doc, to_bson, Document},
    Collection,
};
use rumqttc::{Event, EventLoop, Incoming, Publish};
use tokio::sync::Mutex;

use super::mqtt_messages::{
    ConnectDeviceData, DeviceStatusMQTTMessage, DisconnectDeviceData, ReadBatteryData,
    ReadDeviceErrorData,
};
use crate::{
    auth::get_email_from_client_token,
    backend_core::{
        features::{
            devices_status_feature::{
                models::{BatteryStatus, Component, ComponentStatus, DeviceError},
                web::WebDeviceStatusFeature,
            },
            IotFeature, WebFeature,
        },
        utils::non_primitive_cast,
    },
};

#[derive(Clone)]
pub struct IotDeviceStatusFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<EventLoop>>,
    mongoc: mongodb::Client,
    web_instance: Option<Weak<WebDeviceStatusFeature>>,
    jwt_key: String,
}

impl IotDeviceStatusFeature {}

#[async_trait]
impl IotFeature for IotDeviceStatusFeature {
    fn create(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        jwt_key: String,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        Some(IotDeviceStatusFeature {
            mqttc,
            mongoc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            web_instance: None,
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "devices-status".into()
    }

    fn get_module_name(&self) -> String {
        "devices-status".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    fn set_web_feature_instance<W: WebFeature + 'static>(&mut self, web_instance: Weak<W>)
    where
        Self: Sized,
    {
        self.web_instance = Some(non_primitive_cast(web_instance.clone()).unwrap());
    }

    fn get_web_feature_instance(&self) -> Arc<dyn WebFeature + Send + Sync> {
        self.web_instance.as_ref().unwrap().upgrade().unwrap()
    }

    async fn process_next_mqtt_message(&mut self) {
        let mut mongoc = self.get_mongoc();
        let mut mqtt_event_loop = self.mqtt_event_loop.lock().await;
        if let Ok(Event::Incoming(Incoming::Publish(Publish { payload, .. }))) =
            mqtt_event_loop.poll().await
        {
            if let Some(message) = String::from_utf8(payload.to_vec())
                .ok()
                .and_then(|raw_json| {
                    serde_json::from_str::<DeviceStatusMQTTMessage>(raw_json.as_ref()).ok()
                })
            {
                match message {
                    DeviceStatusMQTTMessage::ReadBattery { token, data } => {
                        if let Some(username) =
                            get_email_from_client_token(self.jwt_key.as_str(), token, &mut mongoc)
                                .await
                        {
                            let device_coll: Collection<Document> =
                                mongoc.default_database().unwrap().collection("devices");
                            for ReadBatteryData { id, value: battery } in data {
                                if let Err(_) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone() }, doc! { "$push": { "battery_logs": to_bson(&BatteryStatus { battery, timestamp: SystemTime::now() }).unwrap() } }, None).await {
                                    eprint!("Failed to process read battery data");
                                }
                            }
                        } else {
                            eprintln!("Invalid token");
                        }
                    }
                    DeviceStatusMQTTMessage::ReadDeviceError { token, data } => {
                        if let Some(username) =
                            get_email_from_client_token(self.jwt_key.as_str(), token, &mut mongoc)
                                .await
                        {
                            let device_coll: Collection<Document> =
                                mongoc.default_database().unwrap().collection("devices");
                            for ReadDeviceErrorData { id, component } in data {
                                if let Err(_) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone() }, doc! { "$push": { "error_logs": to_bson(&DeviceError { id, component, timestamp: SystemTime::now() }).unwrap() } }, None).await {
                                    eprint!("Failed to process read device error data");
                                }
                            }
                        } else {
                            eprintln!("Invalid token");
                        }
                    }
                    DeviceStatusMQTTMessage::ConnectDevice { token, data } => {
                        if let Some(username) =
                            get_email_from_client_token(self.jwt_key.as_str(), token, &mut mongoc)
                                .await
                        {
                            let device_coll: Collection<Document> =
                                mongoc.default_database().unwrap().collection("devices");
                            for ConnectDeviceData {
                                id,
                                component,
                                kind,
                            } in data
                            {
                                match device_coll.find_one(doc! { "id": id, "owner_name": username.clone() }, None).await {
                                    Ok(Some(_)) => {
                                        if let Ok(None) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone(), "components": { "$elemMatch": { "id": component } } }, doc! { "$push": { "components.$.logs": to_bson(&ComponentStatus::Connect { timestamp: SystemTime::now() }).unwrap() } }, None).await {
                                            if let Err(_) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone() }, doc! { "$push": { "components": to_bson(&Component { id: component, kind, logs: vec![ComponentStatus::Connect { timestamp: SystemTime::now() }]  }).unwrap() } }, None).await {
                                                eprintln!("Failed to process connect device data");
                                            }
                                        }
                                    }
                                    Ok(None) => {
                                        if let Err(_) = device_coll.insert_one(doc! { "id": id, "owner_name": username.clone(), "kind": to_bson(&kind).unwrap(), "battery_logs": to_bson(&vec![] as &Vec<BatteryStatus>).unwrap(), "error_logs": to_bson(&vec![] as &Vec<DeviceError>).unwrap(), "components": to_bson(&vec![] as &Vec<Component>).unwrap() }, None).await {
                                            eprintln!("Failed to process connect device data");
                                        } else if let Err(_) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone() }, doc! { "$push": { "components": to_bson(&Component { id, kind, logs: vec![ComponentStatus::Connect { timestamp: SystemTime::now() }]  }).unwrap() } }, None).await {
                                            eprintln!("Failed to process connect device data");
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("Unexpected error while finding devices with id {id}");
                                    }
                                };
                            }
                        } else {
                            eprintln!("Invalid token");
                        }
                    }
                    DeviceStatusMQTTMessage::DisconnectDevice { token, data } => {
                        if let Some(username) =
                            get_email_from_client_token(self.jwt_key.as_str(), token, &mut mongoc)
                                .await
                        {
                            let device_coll: Collection<Document> =
                                mongoc.default_database().unwrap().collection("devices");
                            for DisconnectDeviceData { id, component } in data {
                                match device_coll
                                    .find_one(
                                        doc! { "id": id, "owner_name": username.clone() },
                                        None,
                                    )
                                    .await
                                {
                                    Ok(Some(_)) => {
                                        if let Ok(None) = device_coll.find_one_and_update(doc! { "id": id, "owner_name": username.clone(), "components": { "$elemMatch": { "id": component } } }, doc! { "$push": { "components.$.logs": to_bson(&ComponentStatus::Disconnect { timestamp: SystemTime::now() }).unwrap() } }, None).await {
                                        eprintln!("Cannot disconnect a non-existent component");
                                    }
                                }
                                    Ok(None) => {
                                        eprintln!(
                                            "Device '{}' did not exist for user '{}'",
                                            id,
                                            username.clone()
                                        );
                                    }
                                    Err(_) => {
                                        eprintln!("Unexpected error while finding devices with id '{}' for user '{}", id, username.clone());
                                    }
                                };
                            }
                        } else {
                            eprintln!("Invalid token");
                        }
                    }
                }
            } else {
                eprintln!("Failed to process MQTT message");
            }
        }
    }

    async fn send_message_to_web(&self, message: String) -> String {
        String::from("")
    }
    async fn respond_message_from_web(&self, message: String) -> String {
        String::from("")
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any> {
        self
    }
}
