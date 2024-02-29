use crate::notification::WebNotification;

use super::super::IotFeature;
use async_trait::async_trait;
use rumqttc::Client;

pub struct IotFeatureExample {}

impl IotFeatureExample {}

#[async_trait]
impl IotFeature<(), ()> for IotFeatureExample {
    fn name() -> String {
        "Feature Example".into()
    }

    async fn init(&mut self, rumqttc: &mut Client) {}

    async fn process_iot_message(&mut self, message: ()) {}

    async fn process_web_notification(&mut self, message: WebNotification) {}

    async fn send_iot_command(&mut self, command: ()) {}
}
