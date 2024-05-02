pub struct ClonableWrapper<T> {
    clone: fn(&mut T) -> T,
    data: T,
}

impl<T> ClonableWrapper<T> {
    pub fn clone(&mut self) -> T {
        (self.clone)(&mut self.data)
    }

    pub fn get(&mut self) -> T {
        self.clone()
    }
}

#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $init_mqtt_client:expr, $($feature_module:ident),*) => {{
        let mut web_features = vec![];
        let mut iot_features = vec![];
        let mut toggable_feat_names = vec![];
        use crate::config::JWT_KEY;
        use std::sync::Arc;
        $(
            let mut web_feat = $feature_module::WebFeature::create($mongoc, JWT_KEY.to_owned()).unwrap();
            let mut web_feat_ptr = &mut web_feat as *mut $feature_module::WebFeature;
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let mut iot_feat = $feature_module::IotFeature::create(mqttc, event_loop, $mongoc, JWT_KEY.to_owned()).unwrap();
            let mut iot_feat_ptr = &mut iot_feat as *mut $feature_module::IotFeature; 

            let web_feat_arc = Arc::new(web_feat);
            let iot_feat_arc = Arc::new(iot_feat);
            
            unsafe {
                let mut web_feat_dup = *web_feat_ptr;
                web_feat_dup.set_iot_feature_instance(iot_feat_arc.clone());
                let mut iot_feat_dup = *iot_feat_ptr;
                iot_feat_dup.set_web_feature_instance(web_feat_arc.clone());
                std::mem::forget(web_feat_dup);
                std::mem::forget(iot_feat_dup);
            }

            web_features.push(web_feat_arc as Arc<dyn WebFeature + Send + Sync>);
            iot_features.push(iot_feat_arc as Arc<dyn IotFeature + Send + Sync>);
            if !$feature_module::MUST_ON {
                toggable_feat_names.push($feature_module::IotFeature::name());
            }
        )*
        (web_features, iot_features, toggable_feat_names)
    }};
}
