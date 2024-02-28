use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::time::Duration;

pub struct ClientConfig<'a> {
    pub client_id: &'a str,
    pub broker_hostname: &'a str,
    pub broker_port: u16,
    pub capacity: usize,
    pub keep_alive_sec: u64,
}

pub fn init(config: ClientConfig) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(config.client_id, config.broker_hostname, config.broker_port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive_sec));

    AsyncClient::new(mqttoptions, config.capacity)
}
