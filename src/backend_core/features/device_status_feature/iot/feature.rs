use std::sync::Arc;

use axum::async_trait;
use rumqttc::{Event, Incoming, EventLoop};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::backend_core::{features::IotFeature, utils};
use super::mqtt_messages::DeviceStatusMQTTMessage;
use super::super::notifications::{DeviceStatusIotNotification, DeviceStatusWebNotification};

pub struct IotDeviceStatusFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<EventLoop>>,
    mongoc: mongodb::Client,
    web_tx: Sender<DeviceStatusIotNotification>,
    web_rx: Receiver<DeviceStatusWebNotification>,
}

impl IotDeviceStatusFeature {}

#[async_trait]
impl IotFeature for IotDeviceStatusFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
    ) -> Option<Self>
    where
        Self: Sized,
    {
        Some(IotDeviceStatusFeature {
            mqttc,
            mongoc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            web_tx: utils::non_primitive_cast(web_tx)?,
            web_rx: utils::non_primitive_cast(web_rx)?,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "device-status".into()
    }

    fn get_module_name(&self) -> String {
        "device-status".into()
    }

    fn get_mqttc(&mut self) -> rumqttc::AsyncClient {
        self.mqttc.clone()
    }

    fn get_mongoc(&mut self) -> mongodb::Client {
        self.mongoc.clone()
    }

    async fn process_next_mqtt_message(&mut self) {
        let mut mqtt_event_loop = self.mqtt_event_loop.lock().await;
        match mqtt_event_loop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let payload = String::from_utf8_lossy(&p.payload);
                if let Ok(metrics) = serde_json::from_str::<DeviceStatusMQTTMessage>(&payload) {
                }
            }
            _=> {}
        }
    }
}
