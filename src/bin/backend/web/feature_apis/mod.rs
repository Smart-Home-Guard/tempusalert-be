use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformParameter,
};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
};
use mongodb::{bson::doc, change_stream::session};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{backend_core::models::User, json::Json};

use crate::{
    database_client::{init_database, MONGOC},
    TOGGABLE_FEATURES_NAMES,
};

#[derive(Serialize, JsonSchema)]
struct AllFeaturesResponse {
    feature_names: Vec<String>,
}

async fn get_all_features_handler() -> impl IntoApiResponse {
    (StatusCode::OK, unsafe {
        Json(AllFeaturesResponse {
            feature_names: TOGGABLE_FEATURES_NAMES.clone(),
        })
    })
}

#[derive(Deserialize, Serialize, JsonSchema)]
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

async fn get_all_features_status_handler(
    headers: HeaderMap,
    Path(AllFeatureStatusParams { email }): Path<AllFeatureStatusParams>,
) -> impl IntoApiResponse {
    if headers.get("email").is_none()
        || headers
            .get("email")
            .is_some_and(|value| value != email.as_str())
    {
        return (
            StatusCode::FORBIDDEN,
            Json(AllFeatureStatusResponse {
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
                Json(AllFeatureStatusResponse {
                    feature_status,
                    message: String::from("Successfully"),
                }),
            )
        }
        Ok(None) => (
            StatusCode::BAD_REQUEST,
            Json(AllFeatureStatusResponse {
                feature_status: vec![],
                message: format!("No such user with email '{}'", email.clone()),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AllFeatureStatusResponse {
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
struct UpdateFeatureStatusParams {
    email: String,
}

#[derive(Deserialize, JsonSchema)]
struct UpdateFeatureStatusBody {
    new_feature_status: Vec<FeatureStatus>,
}

async fn update_features_status_handler(
    headers: HeaderMap,
    Path(UpdateFeatureStatusParams { email }): Path<UpdateFeatureStatusParams>,
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
    let mut session = mongoc.start_session(None).await.unwrap();
    session.start_transaction(None).await.unwrap();

    let user_coll = mongoc
        .default_database()
        .unwrap()
        .collection::<User>("users");
    
    let res = match user_coll
        .find_one_with_session(doc! { "email": email.clone() }, None, &mut session)
        .await
    {
        Ok(Some(_)) => {
            for new_feat_status in new_feature_status {
                if new_feat_status.enabled {
                    if let Ok(None) = user_coll.find_one_with_session(doc! { "email": email.clone(), "enabled_features": { "$elemMatch": new_feat_status.name.clone() } }, None, &mut session).await {
                        let _ = user_coll.find_one_and_update_with_session(doc! { "email": email.clone() }, doc! { "$push": { "enabled_features": new_feat_status.name.clone() } }  , None, &mut session).await;
                    }
                } else {
                    let _ = user_coll
                        .find_one_and_update_with_session(
                            doc! { "email": email.clone() },
                            doc! { "$pull": { "enabled_features": new_feat_status.name.clone() } },
                            None,
                            &mut session,
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
    };

    session.commit_transaction().await.unwrap();

    res

}

pub fn features_route() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            get_with(get_all_features_handler, |op| {
                op.description("Get all available features")
                    .tag("Features")
                    .response::<200, Json<AllFeaturesResponse>>()
            }),
        )
        .api_route(
            "/:email",
            get_with(get_all_features_status_handler, |op| {
                op.description("Get all feature status of a given user by email")
                    .tag("Features")
                    .parameter("email", |op: TransformParameter<String>| {
                        op.description("The registered email")
                    })
                    .response::<200, Json<AllFeatureStatusResponse>>()
                    .response::<400, Json<AllFeatureStatusResponse>>()
                    .response::<403, Json<AllFeatureStatusResponse>>()
                    .response::<500, Json<AllFeatureStatusResponse>>()
            })
            .patch_with(update_features_status_handler, |op| {
                op.description("Update the feature status of a given user by email")
                    .tag("Features")
                    .parameter("email", |op: TransformParameter<String>| {
                        op.description("The registered email")
                    })
                    .response::<200, Json<UpdateFeatureStatusResponse>>()
                    .response::<400, Json<UpdateFeatureStatusResponse>>()
                    .response::<403, Json<UpdateFeatureStatusResponse>>()
                    .response::<500, Json<UpdateFeatureStatusResponse>>()
            }),
        )
}
