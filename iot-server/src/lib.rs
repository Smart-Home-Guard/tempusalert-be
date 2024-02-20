use mongodb::Client;
use database_connector::database;

mod features;

use features::Feature;

pub struct Server {
    dbclient: Client,
}

impl Server {
    async fn create(database_url: String, database_name: String) -> Option<Self> {
        match database::init(database_url, database_name).await {
            Ok(client) => Some(Server { dbclient: client}),
            _ => None
        }
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