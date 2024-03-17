use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use mongodb::Collection;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::{database_client::{init_database, MONGOC}, TOGGABLE_FEATURES_NAMES};

#[derive(Serialize, JsonSchema)]
struct FeatureStatus {
    name: String,
    enabled: bool,
}

#[derive(Serialize, JsonSchema)]
struct AllFeaturesResponse {
    feature_status: Vec<FeatureStatus>,
}

async fn get_all_features_handler() -> impl IntoApiResponse {
    let mongoc = MONGOC.get_or_init(init_database).await;
    let toggable_feature_names = unsafe { TOGGABLE_FEATURES_NAMES.clone() };
    let feature_status = vec![];
    for feat_name in toggable_feature_names {
        
    }
}

pub fn features_route() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(get_all_features_handler, |op| {
            op.description("Add subscription for push notification")
                .tag("Push notification")
                .response::<200, Json<PushCredentialResponse>>()
                .response::<500, Json<PushCredentialResponse>>()
        }),
    )
}
