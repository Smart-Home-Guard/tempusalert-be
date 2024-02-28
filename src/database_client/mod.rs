use mongodb::{options::ClientOptions, Client};

pub async fn init(mongodb_uri: String) -> mongodb::error::Result<Client> {
    let client_options = ClientOptions::parse(mongodb_uri).await?;

    let client = Client::with_options(client_options)?;

    println!("✅ Database connected successfully");

    Ok(client)
}
