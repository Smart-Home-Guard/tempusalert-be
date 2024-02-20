use crate::Feature;
use async_trait::async_trait;

pub struct FeatureA {
}

impl FeatureA {

}

#[async_trait]
impl Feature for FeatureA {
    fn name() -> String {
        "FeatureA".into()
    }

    async fn init(&mut self) {
        
    } 

    async fn process_iot_message(&mut self, message: String) {
        
    }

    async fn process_push_notification(&mut self, message: String) {
        
    }

    async fn send_command(&mut self, command: String) {
        
    }
}

