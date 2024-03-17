use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::http::StatusCode;
use mongodb::{bson::doc, Collection};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tempusalert_be::json::Json;

use crate::{
    database_client::{init_database, MONGOC},
    globals::channels::{get_user_publisher, UserEvent, UserEventKind},
    mail::send_mail,
    models::User,
};

use super::utils::hash_password;

#[derive(Deserialize, JsonSchema)]
struct RegisterBody {
    email: String,
    password: String,
}

#[derive(Serialize, JsonSchema)]
pub struct RegisterResponse {
    message: String,
}

async fn register_handler(Json(body): Json<RegisterBody>) -> impl IntoApiResponse {
    let mongoc = MONGOC.get_or_init(init_database).await;
    let user_coll: Collection<User> = mongoc.default_database().unwrap().collection("users");

    if let Ok(Some(User { .. })) = user_coll
        .find_one(doc! { "email": body.email.clone() }, None)
        .await
    {
        (
            StatusCode::BAD_REQUEST,
            Json(RegisterResponse {
                message: String::from("Email already exists"),
            }),
        )
    } else {
        let (hashed_password, salt) = match hash_password(body.password) {
            Some(res) => res,
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterResponse {
                        message: String::from("Failed to hash password"),
                    }),
                )
            }
        };
        let client_id = uuid::Uuid::new_v4().to_string();
        let client_secret = uuid::Uuid::new_v4().to_string();
        if let None = send_mail(
            body.email.clone(),
            String::from("Your gateway credential"),
            format!(
                "
                <p> Please config the given credential at your gateway:
                    <ul>
                        <li> id: {client_id} </li>
                        <li> secret: {client_secret} </li>
                    </ul>
                </p>
                <p>Thanks for choosing our service!<p>

                <footer>
                    <p>Best wishes,<p>
                    <p>Tempusalert team<p>
                </footer>
            "
            ),
        ) {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(RegisterResponse {
                    message: String::from("Failed to send mail"),
                }),
            )
        } else {
            let user = User {
                email: body.email,
                hashed_password,
                client_id: client_id.clone(),
                client_secret,
                salt,
                enabled_features: vec![],
            };
            if let Err(_) = user_coll.insert_one(user, None).await {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(RegisterResponse {
                        message: String::from("Failed to create account"),
                    }),
                )
            } else {
                let user_publisher = get_user_publisher().await;
                let _ = user_publisher.send(UserEvent {
                    kind: UserEventKind::JOIN,
                    client_id: client_id.clone(),
                });
                (
                    StatusCode::OK,
                    Json(RegisterResponse {
                        message: String::from("Registered successfully"),
                    }),
                )
            }
        }
    }
}

pub fn register_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(register_handler, |op| {
            op.description("Registration api")
                .tag("Authentication")
                .response::<200, Json<RegisterResponse>>()
                .response::<400, Json<RegisterResponse>>()
                .response::<500, Json<RegisterResponse>>()
        }),
    )
}
