use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use tempusalert_be::auth::get_email_from_web_token;

use crate::config::JWT_KEY;

pub async fn set_username_from_token_in_request_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let value: Option<&str> = headers.get("jwt").and_then(|value| value.to_str().ok());
    request.headers_mut().remove("email");
    if let Some(jwt) = value {
        request.headers_mut().append(
            "email",
            HeaderValue::from_str(
                get_email_from_web_token(JWT_KEY.as_str(), jwt.to_string())
                    .unwrap_or("".to_string())
                    .as_str(),
            )
            .unwrap(),
        );
    }
    let response = next.run(request).await;
    Ok(response)
}
