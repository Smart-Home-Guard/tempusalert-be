use rumqttc::QoS;
use serde::Serialize;

use crate::errors::AppError;

pub async fn publish_mqtt_message<T: Serialize>(message: T, mqtt_client: rumqttc::AsyncClient, client_id: String, feature_name: String) -> Result<(), AppError> {
    let channel_name = format!("{client_id}/{feature_name}-command");
    mqtt_client.publish(
        channel_name,
        QoS::AtLeastOnce,
        false,
        serde_json::to_value(&message).unwrap().to_string(),
    ).await?;
    Ok(())
}
