use aide::{axum::{routing::get_with, ApiRouter, IntoApiResponse}, transform::TransformParameter};
use axum::{extract::Path, http::{HeaderMap, StatusCode}};
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use tempusalert_be::json::Json;

use crate::{database_client::{init_database, MONGOC}, models::User, TOGGABLE_FEATURES_NAMES};

#[derive(Serialize, JsonSchema)]
struct AllFeaturesResponse {
    feature_names: Vec<String>,
}

async fn get_all_features_handler() -> impl IntoApiResponse {
    (StatusCode::OK, unsafe { Json(AllFeaturesResponse { feature_names: TOGGABLE_FEATURES_NAMES.clone() }) })
}

#[derive(Serialize, JsonSchema)]
struct FeatureStatus {
    name: String,
    enabled: bool,
}

#[derive(Serialize, JsonSchema)]
struct AllFeatureStatusResponse {
    feature_status: Vec<FeatureStatus>,
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct AllFeatureStatusParams {
    email: String,
}

async fn get_all_features_status_handler(headers: HeaderMap, Path(AllFeatureStatusParams { email }): Path<AllFeatureStatusParams>) -> impl IntoApiResponse {
    if headers.get("email").is_none() || headers.get("email").is_some_and(|value| value != email.as_str()) {
        return (StatusCode::FORBIDDEN, Json(AllFeatureStatusResponse { message: String::from("Forbidden"), feature_status: vec![], }));
    }
    let toggable_features = unsafe { TOGGABLE_FEATURES_NAMES.clone() };
    let mongoc = MONGOC.get_or_init(init_database).await;
    match mongoc.default_database().unwrap().collection::<User>("users").find_one(doc! { "email": email.clone() }, None).await {
        Ok(Some(user)) => {
            let User{ enabled_features, .. } = user;
            let mut feature_status = vec![];
            for feat in &enabled_features {
                feature_status.push(FeatureStatus { name: feat.clone(), enabled: true });
            }
            for feat in &toggable_features {
                if !enabled_features.contains(&feat) {
                    feature_status.push(FeatureStatus { name: feat.clone(), enabled: false });
                }
            }
            (StatusCode::OK, Json(AllFeatureStatusResponse { feature_status, message: String::from("Successfully") }) )
        },
        Ok(None) => (StatusCode::BAD_REQUEST, Json(AllFeatureStatusResponse { feature_status: vec![], message: format!("No such user with email '{}'", email.clone() ) })),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(AllFeatureStatusResponse { feature_status: vec![], message: format!("Failed to find the feature status of {}", email.clone() ) })),
    }   
}

pub fn features_route() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(get_all_features_handler, |op| {
                op.description("Get all available features")
                    .tag("Features")
                    .response::<200, Json<AllFeaturesResponse>>()
            }))
        .api_route(
            "/:email",
            get_with(get_all_features_status_handler, |op| {
            op.description("Get all feature status of a given user by email")
                .tag("Features")
                .parameter("email", |op: TransformParameter<String>| op.description("The registered email"))
                .response::<200, Json<AllFeatureStatusResponse>>()
                .response::<400, Json<AllFeatureStatusResponse>>()
                .response::<403, Json<AllFeatureStatusResponse>>()
                .response::<500, Json<AllFeatureStatusResponse>>()
        }))
}
