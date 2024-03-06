use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::http::StatusCode;
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::{auth, json::Json};

use crate::{
    config::JWT_KEY,
    database_client::{init_database, MONGOC},
    parse_env_var,
};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    password: String,
    client_id: String,
    client_secret: String,
}

#[derive(Deserialize, JsonSchema)]
struct IotAuthBody {
    client_id: String,
    client_secret: String,
}

#[derive(Serialize)]
struct IotClientClaim {
    client_id: String,
    nonce: String,
}

#[derive(Serialize, JsonSchema)]
enum Token {
    Some(String),
    None,
}

async fn iot_auth_handler(Json(body): Json<IotAuthBody>) -> impl IntoApiResponse {
    let mongoc = MONGOC.get_or_init(init_database).await;
    let user_coll: Collection<User> = mongoc
        .database(parse_env_var::<String>("MONGO_INITDB_DATABASE").as_str())
        .collection("users");

    if let Some(_) = user_coll
        .find_one(
            doc! { "client_id": body.client_id.clone(), "client_secret": body.client_secret },
            None,
        )
        .await
        .ok()
    {
        let client_claim = IotClientClaim {
            client_id: body.client_id,
            nonce: uuid::Uuid::new_v4().into(),
        };
        let token = auth::sign_jwt(JWT_KEY.as_str(), &client_claim);

        if let Some(token) = token {
            (StatusCode::OK, Json(Token::Some(token)))
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(Token::None))
        }
    } else {
        (StatusCode::BAD_REQUEST, Json(Token::None))
    }
}

pub fn iot_auth_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/iot",
        post_with(iot_auth_handler, |op| {
            op.description("Iot authentication api")
                .tag("Authentication")
                .response::<200, Json<String>>()
                .response::<400, Json<String>>()
                .response::<500, Json<String>>()
        }),
    )
}
