use aide::operation::OperationIo;
use axum::response::IntoResponse;
use axum_jsonschema::JsonSchemaRejection;
use axum_macros::FromRequest;
use serde::Serialize;

use crate::errors::{AppError, BadRequest};

#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(AppError))]
#[aide(
    input_with = "axum::Json<T>",
    output_with = "axum::Json<T>",
    json_schema
)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

impl From<JsonSchemaRejection> for AppError {
    fn from(rejection: JsonSchemaRejection) -> Self {
        match rejection {
            JsonSchemaRejection::Json(_) => Self::BadRequest(BadRequest {}),
            JsonSchemaRejection::Serde(_) => Self::BadRequest(BadRequest {}),
            JsonSchemaRejection::Schema(_) => Self::BadRequest(BadRequest {}),
        }
    }
}
