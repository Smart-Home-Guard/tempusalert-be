use mongodb::{bson::Document, options::ClientOptions, Client, Collection};
use dotenv::dotenv;

#[derive(Debug)]
pub struct DB {
    pub collection: Collection<Document>,
}

impl DB {
    pub async fn init() -> mongodb::error::Result<Client> {
        dotenv().ok();
        let mongodb_uri: String = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let database_name: String =
            std::env::var("MONGO_INITDB_DATABASE").expect("MONGO_INITDB_DATABASE must be set.");

        let mut client_options = ClientOptions::parse(mongodb_uri).await?;
        client_options.app_name = Some(database_name.to_string());

        let client = Client::with_options(client_options)?;

        println!("✅ Database connected successfully");

        Ok(client)
    }
}
