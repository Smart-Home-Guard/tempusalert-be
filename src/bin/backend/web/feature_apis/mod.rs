use aide::axum::{routing::get_with, ApiRouter, IntoApiResponse};
use axum::{
    extract::Query,
    http::{HeaderMap, StatusCode},
};
use mongodb::bson::doc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{backend_core::models::User, json::Json};

use crate::{
    database_client::{init_database, MONGOC},
    TOGGABLE_FEATURES_NAMES,
};

#[derive(Serialize, JsonSchema)]
enum AllFeaturesResponse {
    FeatureQuery {
        feature_names: Vec<String>,
    },
    FeatureStatusResponse {
        feature_status: Vec<FeatureStatus>,
        message: String,
    },
}

#[derive(Deserialize, Serialize, JsonSchema)]
struct FeatureStatus {
    name: String,
    enabled: bool,
}

#[derive(Deserialize, JsonSchema)]
struct AllFeatureStatusQuery {
    email: Option<String>,
}

async fn get_all_features_status_handler(
    headers: HeaderMap,
    Query(AllFeatureStatusQuery { email }): Query<AllFeatureStatusQuery>,
) -> impl IntoApiResponse {
    if email.is_none() {
        return (StatusCode::OK, unsafe {
            Json(AllFeaturesResponse::FeatureQuery {
                feature_names: TOGGABLE_FEATURES_NAMES.clone(),
            })
        });
    }
    let email = email.unwrap();
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(AllFeaturesResponse::FeatureStatusResponse {
                message: String::from("Forbidden"),
                feature_status: vec![],
            }),
        );
    }
    let toggable_features = unsafe { TOGGABLE_FEATURES_NAMES.clone() };
    let mongoc = MONGOC.get_or_init(init_database).await;
    match mongoc
        .default_database()
        .unwrap()
        .collection::<User>("users")
        .find_one(doc! { "email": email.clone() }, None)
        .await
    {
        Ok(Some(user)) => {
            let User {
                enabled_features, ..
            } = user;
            let mut feature_status = vec![];
            for feat in &enabled_features {
                feature_status.push(FeatureStatus {
                    name: feat.clone(),
                    enabled: true,
                });
            }
            for feat in &toggable_features {
                if !enabled_features.contains(&feat) {
                    feature_status.push(FeatureStatus {
                        name: feat.clone(),
                        enabled: false,
                    });
                }
            }
            (
                StatusCode::OK,
                Json(AllFeaturesResponse::FeatureStatusResponse {
                    feature_status,
                    message: String::from("Successfully"),
                }),
            )
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(AllFeaturesResponse::FeatureStatusResponse {
                feature_status: vec![],
                message: format!("No such user with email '{}'", email.clone()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AllFeaturesResponse::FeatureStatusResponse {
                feature_status: vec![],
                message: format!("Failed to find the feature status of {}", email.clone()),
            }),
        ),
    }
}

#[derive(Serialize, JsonSchema)]
struct UpdateFeatureStatusResponse {
    message: String,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateFeatureStatusQuery {
    email: String,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateFeatureStatusBody {
    new_feature_status: Vec<FeatureStatus>,
}

async fn update_features_status_handler(
    headers: HeaderMap,
    Query(UpdateFeatureStatusQuery { email }): Query<UpdateFeatureStatusQuery>,
    Json(UpdateFeatureStatusBody { new_feature_status }): Json<UpdateFeatureStatusBody>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(UpdateFeatureStatusResponse {
                message: String::from("Forbidden"),
            }),
        );
    }

    let mongoc = MONGOC.get_or_init(init_database).await;
    let user_coll = mongoc
        .default_database()
        .unwrap()
        .collection::<User>("users");
    match user_coll
        .find_one(doc! { "email": email.clone() }, None)
        .await
    {
        Ok(Some(_)) => {
            for new_feat_status in new_feature_status {
                if new_feat_status.enabled {
                    if let Ok(None) = user_coll.find_one(doc! { "email": email.clone(), "enabled_features": { "$elemMatch": new_feat_status.name.clone() } }, None).await {
                        let _ = user_coll.find_one_and_update(doc! { "email": email.clone() }, doc! { "$push": { "enabled_features": new_feat_status.name.clone() } }  , None).await;
                    }
                } else {
                    let _ = user_coll
                        .find_one_and_update(
                            doc! { "email": email.clone() },
                            doc! { "$pull": { "enabled_features": new_feat_status.name.clone() } },
                            None,
                        )
                        .await;
                }
            }
            (
                StatusCode::OK,
                Json(UpdateFeatureStatusResponse {
                    message: String::from("Successfully"),
                }),
            )
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(UpdateFeatureStatusResponse {
                message: format!("No such user with email '{}'", email.clone()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(UpdateFeatureStatusResponse {
                message: format!("Failed to find the feature status of {}", email.clone()),
            }),
        ),
    }
}

pub fn features_route() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        get_with(get_all_features_status_handler, |op| {
            op.description("Get all feature status")
                .tag("Features")
                .response::<200, Json<AllFeaturesResponse>>()
                .response::<400, Json<AllFeaturesResponse>>()
                .response::<403, Json<AllFeaturesResponse>>()
                .response::<500, Json<AllFeaturesResponse>>()
        })
        .patch_with(update_features_status_handler, |op| {
            op.description("Update the feature status of a user by email if given")
                .tag("Features")
                .response::<200, Json<UpdateFeatureStatusResponse>>()
                .response::<400, Json<UpdateFeatureStatusResponse>>()
                .response::<403, Json<UpdateFeatureStatusResponse>>()
                .response::<500, Json<UpdateFeatureStatusResponse>>()
        }),
    )
}
