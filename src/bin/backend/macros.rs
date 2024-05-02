#[macro_export]
macro_rules! create_features {
    ($mongoc:expr, $init_mqtt_client:expr, $($feature_module:ident),*) => {{
        let mut web_features = vec![];
        let mut iot_features = vec![];
        let mut toggable_feat_names = vec![];
        use crate::config::JWT_KEY;
        use std::sync::Arc; 
        use crate::clonable_wrapper::ClonableWrapper;
        use crate::types::*;
        $(
            let mut web_feat = $feature_module::WebFeature::create($mongoc, JWT_KEY.to_owned()).unwrap();
            let web_feat_ptr = &mut web_feat as *mut $feature_module::WebFeature;
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let mut iot_feat = $feature_module::IotFeature::create(mqttc, event_loop, $mongoc, JWT_KEY.to_owned()).unwrap();
            let iot_feat_ptr = &mut iot_feat as *mut $feature_module::IotFeature; 

            let web_feat_arc = Arc::new(web_feat);
            let iot_feat_arc = Arc::new(iot_feat);
            
            unsafe {
                let mut web_feat_dup = std::ptr::read(web_feat_ptr);
                web_feat_dup.set_iot_feature_instance(Arc::downgrade(&iot_feat_arc));
                let mut iot_feat_dup = std::ptr::read(iot_feat_ptr);
                iot_feat_dup.set_web_feature_instance(Arc::downgrade(&web_feat_arc));
                std::mem::forget(web_feat_dup);
                std::mem::forget(iot_feat_dup);
            }

            let web_clonable_wrapper: ClonableWrapper<WebFeatureDyn> = ClonableWrapper::create(
                Box::new(|e| Box::new($feature_module::WebFeature::clone((&e as &dyn std::any::Any).downcast_ref::<$feature_module::WebFeature>().unwrap()))),
                web_feat_arc,
            );

            let iot_clonable_wrapper: ClonableWrapper<IotFeatureDyn> = ClonableWrapper::create(
                Box::new(|e| Box::new($feature_module::IotFeature::clone((&e as &dyn std::any::Any).downcast_ref::<$feature_module::IotFeature>().unwrap()))),
                iot_feat_arc,
            );

            web_features.push(web_clonable_wrapper);
            iot_features.push(iot_clonable_wrapper);
            if !$feature_module::MUST_ON {
                toggable_feat_names.push($feature_module::IotFeature::name());
            }
        )*
        (web_features, iot_features, toggable_feat_names)
    }};
}
