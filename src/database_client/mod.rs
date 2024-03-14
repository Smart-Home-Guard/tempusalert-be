use mongodb::{options::{ClientOptions, Credential, ServerAddress}, Client};

pub struct MongocConfig {
    pub server_hostname: String,
    pub server_port: u16,
    pub username: String,
    pub password: String,
    pub default_db: String,
    pub auth_source: String,
}

pub async fn init(config: MongocConfig) -> mongodb::error::Result<Client> {
    let client_options = ClientOptions::builder()
        .direct_connection(true)
        .hosts(vec![ServerAddress::Tcp { host: config.server_hostname, port: Some(config.server_port) }])
        .credential(Credential::builder()
            .username(config.username)
            .password(config.password)
            .source(config.auth_source)
            .build()
        )
        .default_database(config.default_db)
        .build();

    let client = Client::with_options(client_options)?;

    println!("✅ Database connected successfully");

    Ok(client)
}
