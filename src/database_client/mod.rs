use mongodb::{options::{ClientOptions, Credential, ServerAddress}, Client};

pub struct MongocReplicaMember {
    pub hostname: String,
    pub port: u16,
}

pub struct MongocConfig {
    pub replica_members: Vec<MongocReplicaMember>,
    pub replica_set_name: String,
    pub username: String,
    pub password: String,
    pub default_db: String,
    pub auth_source: String,
}

pub async fn init(config: MongocConfig) -> mongodb::error::Result<Client> {
    let replica_members: Vec<ServerAddress> = config.replica_members.iter().map(|m| ServerAddress::Tcp { host: m.hostname.clone(), port: Some(m.port) }).collect();
    let client_options = ClientOptions::builder()
        .repl_set_name(config.replica_set_name)
        .hosts(replica_members)
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
