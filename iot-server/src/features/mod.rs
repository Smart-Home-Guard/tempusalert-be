mod featureA;

use async_trait::async_trait;

#[async_trait]
pub trait Feature {
    fn name() -> String;
    async fn init(&mut self);
    async fn process_iot_message(&mut self, message: String);
    async fn process_push_notification(&mut self, message: String);
    async fn send_command(&mut self, command: String);
}