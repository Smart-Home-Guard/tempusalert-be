use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::http::StatusCode;
use mongodb::{
    bson::{doc, to_bson, Document},
    Collection,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::json::Json;

use crate::{
    database_client::{init_database, MONGOC},
    models::PushCredential,
};

#[derive(Serialize, Deserialize, JsonSchema)]
struct PushCredentialBody {
    credential: PushCredential,
    username: String,
}

#[derive(Serialize, JsonSchema)]
struct PushCredentialResponse {
    message: String,
}

async fn push_handler(Json(body): Json<PushCredentialBody>) -> impl IntoApiResponse {
    let mongoc = MONGOC.get_or_init(init_database).await;
    let push_cred_coll: Collection<Document> = mongoc
        .default_database()
        .unwrap()
        .collection("push_credentials");
    if let Err(_) = push_cred_coll.insert_one(doc! { "endpoint": body.credential.endpoint, "key": to_bson(&body.credential.key).unwrap(), "username": body.username }, None).await {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(PushCredentialResponse{ message: String::from("Failed to add subscription") }))
    } else {
        (StatusCode::OK, Json(PushCredentialResponse{ message: String::from("Successfully add user subscription") }))
    }
}

pub fn push_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(push_handler, |op| {
            op.description("Add subscription for push notification")
                .tag("Push notification")
                .response::<200, Json<PushCredentialResponse>>()
                .response::<500, Json<PushCredentialResponse>>()
        }),
    )
}
