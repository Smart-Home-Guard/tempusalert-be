use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::time::Duration;

pub struct ClientConfig {
    pub client_id: String,
    pub broker_hostname: String,
    pub broker_port: u16,
    pub capacity: usize,
    pub keep_alive_sec: u64,
}

#[tokio::main]
pub async fn init(config: ClientConfig) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(config.client_id, config.broker_hostname, config.broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive_sec));

    AsyncClient::new(mqttoptions, config.capacity)
}
