use std::thread;
use std::time::Duration;

use rumqttc::{Client, Event, Incoming, MqttOptions, QoS};
use serde_json::Value;

pub fn handle_topic_a() {
    let client_id = "9f5a3bbd-907d-4a9b-9de7-6b8021843b37";
    let client_telemetry_topic = format!("{}/telemetry", client_id);
    let server_command_topic = format!("{}/commands", client_id);
    let client_name = format!("{}/nightlight_server", client_id);

    let mut mqttoptions = MqttOptions::new(client_name, "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    client
        .subscribe(client_telemetry_topic, QoS::AtMostOnce)
        .unwrap();

    thread::spawn(move || {
        // Simulating telemetry data in Rust
        // In your actual implementation, this would be replaced by MQTT message handling
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Iterate to poll the eventloop for connection progress
    for notification in connection.iter() {
        match notification {
            Ok(Event::Incoming(Incoming::Publish(p))) => {
                let payload = String::from_utf8_lossy(&p.payload);
                if let Ok(telemetry) = serde_json::from_str::<Value>(&payload) {
                    println!("Message received: {}", telemetry);
                    if let Some(light) = telemetry.get("light").and_then(Value::as_u64) {
                        let command = serde_json::json!({"led_on": light < 300});
                        println!("Sending message: {}", command);
                        client
                            .publish(
                                &server_command_topic,
                                QoS::AtLeastOnce,
                                false,
                                command.to_string().into_bytes(),
                            )
                            .unwrap();
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error: {:?}", e);
                break;
            }
        }
    }
}