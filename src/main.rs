use dotenv::dotenv;
use tempusalert_be::repositories::DB;

#[tokio::main]
pub async fn main() {
    dotenv().ok();
    DB::init().await.unwrap();
}