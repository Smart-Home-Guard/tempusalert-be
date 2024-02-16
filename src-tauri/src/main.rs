// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dotenv::dotenv;
use tempusalert::{database::DB, handler};

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    dotenv().ok();
    DB::init().await.unwrap();

    handler::run();
}
