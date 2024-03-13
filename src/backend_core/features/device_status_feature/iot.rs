use std::sync::Arc;

use axum::async_trait;
use rumqttc::{Event, Incoming, EventLoop};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::backend_core::{features::IotFeature, utils};

use super::notifications::{DeviceStatusIotNotification, DeviceStatusWebNotification};

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

    async fn run_loop(&mut self) {
        let event_loop = &self.mqtt_event_loop;
        println!("a");

        while let Ok(notification) = event_loop.lock().await.poll().await {
            match notification {
                Event::Incoming(Incoming::Publish(p)) => {
                    let payload = String::from_utf8_lossy(&p.payload);
                    if let Ok(metrics) = serde_json::from_str::<DeviceStatusMetric>(&payload) {
                        println!("Message received: {:?}", metrics);
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct DeviceStatusMetric {
    kind: DeviceStatusFunction,
    data: Vec<DeviceData>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum DeviceStatusFunction {
    ReadBaterry,
    ReadError,
    ConnectDevice,
    DisconnectDevice,
}

#[derive(Debug, Deserialize, Serialize)]
struct DeviceData {
    id: u8,
    value: Option<u8>,
    component: Option<u8>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use mongodb::Client;
    use rumqttc::{AsyncClient, MqttOptions, QoS};
    use tokio::{sync::mpsc, task, time};

    use super::*;

    #[tokio::test]
    async fn test_run_loop_publish() {
        let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        client
            .subscribe("35b803ff-0537-4f16-aab0-90421a06026a", QoS::AtMostOnce)
            .await
            .unwrap();

        let client_publish = client.clone();

        task::spawn(async move {
            for i in 0..5 {
                client_publish
                    .publish(
                        "35b803ff-0537-4f16-aab0-90421a06026a",
                        QoS::AtLeastOnce,
                        false,
                        serde_json::to_string(&DeviceStatusMetric {
                            kind: DeviceStatusFunction::ReadBaterry,
                            data: vec![DeviceData {
                                id: i,
                                value: Some(i),
                                component: Some(i),
                            }],
                        })
                        .unwrap(),
                    )
                    .await
                    .unwrap();
                time::sleep(Duration::from_millis(100)).await;
            }
        });

        let (_, web_rx) = mpsc::channel::<DeviceStatusWebNotification>(64);
        let (web_tx, _) = mpsc::channel::<DeviceStatusIotNotification>(64);

        let uri = "mongodb://user:password123@localhost:6000/tempusalert?authSource=admin";
        let mongo_client = Client::with_uri_str(uri).await.unwrap();

        let feature =
            IotDeviceStatusFeature::create(client, eventloop, mongo_client, web_tx, web_rx);

        feature.unwrap().run_loop().await;
    }
}
