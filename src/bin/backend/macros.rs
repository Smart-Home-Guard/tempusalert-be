
#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $mqttc:expr, $($WebFeature:ident, $IotFeature:ident),*) => {{
        let web_features = vec![];
        let iot_features = vec![];
        use tokio::sync::mpsc;
        $(
            let (web_rx, iot_tx) = mpsc::channel(64);
            let (iot_rx, web_tx) = mpsc::channel(64);
            web_features.push($WebFeature::create($mongoc, iot_rx, iot_tx));
            iot_features.push($WebFeature::create($mongoc, $mqttc, web_rx, web_tx));
        )*
        (web_features, iot_features)
    }};
}