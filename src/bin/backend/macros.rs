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
            let web_feat = $feature_module::WebFeature::create($mongoc, JWT_KEY.to_owned()).unwrap();
            let (mqttc, event_loop) = $init_mqtt_client($feature_module::IotFeature::name().as_str()).await;
            let iot_feat = $feature_module::IotFeature::create(mqttc, event_loop, $mongoc, JWT_KEY.to_owned()).unwrap();

            let mut web_feat_arc = Arc::new(web_feat);
            let mut iot_feat_arc = Arc::new(iot_feat);
            
            let web_feat_ptr = Arc::get_mut(&mut web_feat_arc).unwrap() as *mut $feature_module::WebFeature;
            let iot_feat_ptr = Arc::get_mut(&mut iot_feat_arc).unwrap() as *mut $feature_module::IotFeature;
            
            unsafe {
                (*web_feat_ptr).set_iot_feature_instance(Arc::downgrade(&iot_feat_arc));
                (*iot_feat_ptr).set_web_feature_instance(Arc::downgrade(&web_feat_arc));
            }

            let web_clonable_wrapper: ClonableWrapper<WebFeatureDyn> = ClonableWrapper::create(
                Box::new(|e: Arc<WebFeatureDyn>| {
                    let e_any = e.into_any();
                    Box::new($feature_module::WebFeature::clone(e_any.downcast_ref::<$feature_module::WebFeature>().unwrap()))
                }),
                web_feat_arc,
            );

            let iot_clonable_wrapper: ClonableWrapper<IotFeatureDyn> = ClonableWrapper::create(
                Box::new(|e: Arc<IotFeatureDyn>| {
                    let e_any = e.into_any();
                    Box::new($feature_module::IotFeature::clone(e_any.downcast_ref::<$feature_module::IotFeature>().unwrap()))
                }),
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
