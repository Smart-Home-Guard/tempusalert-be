use axum::async_trait;
use rumqttc::{Event, Incoming, Publish};
use std::sync::Arc;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::backend_core::{
    features::{
        remote_control_feature::{IotNotification, WebNotification}, IotFeature
    },
    utils::non_primitive_cast
};

pub struct IotRemoteControlFeature {
    mqttc: rumqttc::AsyncClient,
    mqtt_event_loop: Arc<Mutex<rumqttc::EventLoop>>,
    mongoc: mongodb::Client,
    web_tx: Sender<IotNotification>,
    web_rx: Receiver<WebNotification>,
    jwt_key: String,
}

impl IotRemoteControlFeature {}

#[async_trait]
impl IotFeature for IotRemoteControlFeature {
    fn create<I: 'static, W: 'static>(
        mqttc: rumqttc::AsyncClient,
        mqtt_event_loop: rumqttc::EventLoop,
        mongoc: mongodb::Client,
        web_tx: Sender<I>,
        web_rx: Receiver<W>,
        jwt_key: String,
    ) -> Option<Self> {
        Some(IotRemoteControlFeature {
            mqttc,
            mqtt_event_loop: Arc::new(Mutex::new(mqtt_event_loop)),
            mongoc: mongoc.clone(),
            web_tx: non_primitive_cast(web_tx)?,
            web_rx: non_primitive_cast(web_rx)?,
            jwt_key,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "remote-control".into()
    }

    fn get_module_name(&self) -> String {
        "remote-control".into()
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
        }
    }
    async fn process_next_web_push_message(&mut self) {
        
    }
}
