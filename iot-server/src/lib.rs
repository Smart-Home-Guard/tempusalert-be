mod features;

use database_connector::database;
use rumqttc::MqttOptions;
use features::Feature;

pub struct ServerConfig {
    mongo_url: String,
    server_id: String,
    mqtt_url: String,
    mqtt_cap: usize,
}

pub struct Server {
    mongo_client: mongodb::Client,
    mqtt_client: rumqttc::Client,
    mqtt_connection: rumqttc::Connection,
}

impl Server {
    async fn create(config: ServerConfig) -> Option<Self> {
        match database::init(config.mongo_url).await {
            Ok(mongo_client) => Some(mongo_client),
            _ => None
        }.and_then( |mongo_client| {
            let options = rumqttc::MqttOptions::parse_url(config.mqtt_url)?;
            let (mut client, mut connection) = rumqttc::Client::new(options, config.mqtt_cap);
            Some(Server { mongo_client, mqtt_client: client, mqtt_connection: connection })
        })
    }

    async fn register(&mut self, mut module: impl Feature) -> &mut Self {
        module.init();
        self
    }

    async fn run(&mut self) {
        // receive message
        // delegate message
    }
}