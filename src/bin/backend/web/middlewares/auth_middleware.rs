use axum::{
    extract::Request,
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::{headers::Cookie, TypedHeader};
use tempusalert_be::auth::get_email_from_token;

use crate::config::JWT_KEY;

pub async fn set_username_from_token_in_request_middleware(
    TypedHeader(cookie): TypedHeader<Cookie>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let value = cookie.get("JWT");
    if let Some(jwt) = value {
        request.headers_mut().append(
            "email",
            HeaderValue::from_str(
                get_email_from_token(JWT_KEY.as_str(), jwt.to_string())
                    .unwrap_or("".to_string())
                    .as_str(),
            )
            .unwrap(),
        );
    } else {
        request
            .headers_mut()
            .append("email", HeaderValue::from_str("").unwrap());
    }
    let response = next.run(request).await;
    Ok(response)
}
