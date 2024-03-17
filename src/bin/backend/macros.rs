#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $init_mqtt_client:expr, $($feature_module:ident),*) => {{
        let mut web_features = vec![];
        let mut iot_features = vec![];
        let mut toggable_feat_names = vec![];
        use tokio::sync::mpsc;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use crate::config::JWT_KEY;
        $(
            let (iot_rx, web_tx) = mpsc::channel::<$feature_module::WebNotification>(64);
            let (web_rx, iot_tx) = mpsc::channel::<$feature_module::IotNotification>(64);
            let web_feat = Arc::new(Mutex::new($feature_module::WebFeature::create($mongoc, iot_rx, iot_tx, JWT_KEY.to_owned()).unwrap())) as Arc<Mutex<dyn WebFeature + Send + Sync>>;
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let iot_feat = Arc::new(Mutex::new($feature_module::IotFeature::create(mqttc, event_loop, $mongoc, web_rx, web_tx, JWT_KEY.to_owned()).unwrap())) as Arc<Mutex<dyn IotFeature + Send + Sync>>;
            web_features.push(web_feat);
            iot_features.push(iot_feat);
            if !$feature_module::MUST_ON {
                toggable_feat_names.push($feature_module::IotFeature::name());
            }
        )*
        (web_features, iot_features, toggable_feat_names)
    }};
}
