use aide::axum::{routing::post_with, ApiRouter, IntoApiResponse};
use axum::{http::{HeaderName, StatusCode}, response::AppendHeaders};
use schemars::JsonSchema;
use serde::Serialize;
use tempusalert_be::json::Json;

#[derive(Serialize, JsonSchema)]
struct LogoutResponse {
    message: String,
}

async fn logout_handler() -> impl IntoApiResponse {
    (
        StatusCode::OK,
        AppendHeaders([(HeaderName::from_static("jwt"), String::new()), (HeaderName::from_static("loggedin"), String::new())]),
        Json(LogoutResponse { message: String::from("Logout successfully"), }),
    )
}

pub fn logout_routes() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(logout_handler, |op| {
            op.description("Logout API")
                .tag("Authentication")
                .response::<200, Json<LogoutResponse>>()
        }),
    )
}
