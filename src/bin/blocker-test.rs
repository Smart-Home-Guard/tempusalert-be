use std::time::Duration;

use rumqttc::QoS;
use tempusalert_be::mqtt_client::{self, ClientConfig};
use tokio::{task, time};

#[tokio::main]
async fn main() {
    let config = ClientConfig { 
      client_id: "rum_mqtt-async",
      broker_hostname: "0.0.0.0",
      broker_port: 1883,
      capacity:10,
      keep_alive_sec:5
    };

    let (mut client, mut eventloop) = mqtt_client::init(config);
    client
        .subscribe("hello/rumqtt", QoS::AtMostOnce)
        .await
        .unwrap();

    task::spawn(async move {
        for i in 0..10 {
            client
                .publish("hello/rumqtt", QoS::AtLeastOnce, false, vec![i; i as usize])
                .await
                .unwrap();
            time::sleep(Duration::from_millis(100)).await;
        }
    });

    loop {
        let notification = eventloop.poll().await.unwrap();
        println!("Received = {:?}", notification);
    }
}
