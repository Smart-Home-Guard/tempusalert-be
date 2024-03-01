use std::sync::Arc;

use crate::{
    backend_core::features::WebFeature,
    message::{IotCommand, IotMessage, IotNotification, WebNotification},
};

use super::{super::IotFeature, WebExampleFeature};
use async_trait::async_trait;
use rumqttc::Client;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

pub struct IotExampleFeature;

impl IotExampleFeature {}

#[async_trait]
impl IotFeature for IotExampleFeature {
    fn create(
        web_rx: Arc<Mutex<Receiver<WebNotification>>>,
        web_tx: &mut Sender<IotNotification>,
    ) -> Self {
        IotExampleFeature
    }

    fn set_receiver(&mut self, web_feat: Arc<dyn WebFeature>) {}

    async fn send(&mut self, notif: IotNotification) {}

    async fn recv(&mut self) -> WebNotification {
        WebNotification::None
    }

    fn name(&mut self) -> String {
        "Feature Example".into()
    }

    async fn init(&mut self, rumqttc: &mut Client) {}

    async fn process_iot_message(&mut self, message: IotMessage) {}

    async fn process_web_notification(&mut self, message: WebNotification) {}

    async fn send_iot_command(&mut self, command: IotCommand) {}
}
