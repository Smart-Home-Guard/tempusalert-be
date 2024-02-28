use mongodb::{bson::Document, options::ClientOptions, Client, Collection};

use crate::web_core::types::AppResult;

#[derive(Debug)]
pub struct DB {
    pub collection: Collection<Document>,
}

impl DB {
    pub async fn init() -> AppResult<Self> {
        let mongodb_uri: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let database_name: String =
            std::env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set.");
        let mongodb_note_collection: String =
            std::env::var("MONGODB_COLLECTION").expect("MONGODB_COLLECTION must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri).await?;
        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)?;
        let database = client.database(database_name.as_str());

        let collection = database.collection::<Document>(mongodb_note_collection.as_str());

        println!("✅ Database connected successfully");

        Ok(Self { collection })
    }
}
