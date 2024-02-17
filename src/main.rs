use dotenv::dotenv;
use tempusalert_be::repositories::DB;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    DB::init().await.unwrap();
}