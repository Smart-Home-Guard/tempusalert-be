use rumqttd::{Broker, Config};
use std::{file, path::Path};

#[tokio::main]
async fn main() {
    let this_file = file!();
    let path = Path::new(this_file);
    let config_path = path.parent().unwrap().join("rumqttd.toml");

    let builder = tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .expect("initialized subscriber succesfully");

    let config = config::Config::builder()
        .add_source(config::File::with_name(config_path.to_str().unwrap()))
        .build()
        .unwrap();
    let config: Config = config.try_deserialize().unwrap();

    let mut broker = Broker::new(config);

    broker.start().unwrap();
}
