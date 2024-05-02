#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $init_mqtt_client:expr, $($feature_module:ident),*) => {{
        let mut web_features = vec![];
        let mut iot_features = vec![];
        let mut toggable_feat_names = vec![];
        use crate::config::JWT_KEY;
        use std::sync::Arc;
        $(
            let web_feat = $feature_module::WebFeature::create($mongoc, JWT_KEY.to_owned()).unwrap();
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let iot_feat = $feature_module::IotFeature::create(mqttc, event_loop, $mongoc, JWT_KEY.to_owned()).unwrap();
            let web_feat_arc = Arc::new(web_feat) as Arc<dyn WebFeature + Send + Sync>;
            let iot_feat_arc = Arc::new(iot_feat) as Arc<dyn IotFeature + Send + Sync>;
            iot_feat.set_web_feature_instance(web_feat_arc.clone());
            web_feat.set_iot_feature_instance(iot_feat_arc.clone());
            web_features.push(web_feat_arc);
            iot_features.push(iot_feat_arc);
            if !$feature_module::MUST_ON {
                toggable_feat_names.push($feature_module::IotFeature::name());
            }
        )*
        (web_features, iot_features, toggable_feat_names)
    }};
}
