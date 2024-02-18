use dotenv::dotenv;
use futures::FutureExt;
use tracing::info;

use tempusalert_be::{
    configs::{self, AppConfig},
    constants::CONFIG,
    repositories,
    routes::{create_router_app, AppState},
    types::AppResult,
    utils,
};

pub struct AppServer {
    pub state: AppState,
    tcp: tokio::net::TcpListener,
}
impl AppServer {
    pub async fn new(mut config: AppConfig) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.server.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        tracing::info!("The server is listening on: {addr}");
        config.server.port = addr.port();
        let state = AppState::new(config).await?;
        Ok(Self { state, tcp })
    }

    pub async fn run(self) -> AppResult<()> {
        let router = create_router_app(self.state);
        axum::serve(self.tcp, router).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    repositories::DB::init().await.unwrap();

    let _file_appender_guard = configs::tracing::init().unwrap();
    info!("The initialization of Tracing was successful.");

    let config = CONFIG.clone();
    info!("Reading the config file was successful.");

    info!("Create a new server.");
    let server = AppServer::new(config).await.unwrap();

    info!("Run the server.");
    utils::task::join_all(vec![(true, server.run().boxed())])
        .await
        .unwrap();
}
