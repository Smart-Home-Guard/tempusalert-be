
#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $init_mqtt_client:expr, $($feature_module:ident),*) => {{
        let mut web_features = vec![];
        let mut iot_features = vec![];
        use tokio::sync::mpsc;
        $(
            let (iot_rx, web_tx) = mpsc::channel::<$feature_module::WebNotification>(64);
            let (web_rx, iot_tx) = mpsc::channel::<$feature_module::IotNotification>(64);
            let web_feat = $feature_module::WebFeature::create($mongoc, iot_rx, iot_tx); 
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let iot_feat = $feature_module::IotFeature::create(mqttc, event_loop, $mongoc, web_rx, web_tx);
            web_features.push(Box::new(web_feat) as Box<dyn WebFeature + Send + Sync>);
            iot_features.push(Box::new(iot_feat) as Box<dyn IotFeature + Send + Sync>);
        )*
        (web_features, iot_features)
    }};
}