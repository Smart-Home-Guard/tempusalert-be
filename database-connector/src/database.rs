use mongodb::{options::ClientOptions, Client};

pub async fn init(mongodb_uri: String, database_name: String) -> mongodb::error::Result<Client> {
    let mut client_options = ClientOptions::parse(mongodb_uri).await?;
    client_options.app_name = Some(database_name);

    let client = Client::with_options(client_options)?;

    println!("✅ Database connected successfully");

    Ok(client)
}
