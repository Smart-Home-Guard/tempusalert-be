use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::result::Result;
use utoipa::openapi::{path::OperationBuilder, PathItem, PathItemType};
use utoipa::{ToResponse, ToSchema};

use super::{Feature, SwaggerMeta};
use crate::errors::AppError;

#[derive(Serialize, ToSchema, ToResponse)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

pub struct FeatureExample;

impl FeatureExample {
    pub async fn health_check() -> Result<Json<GenericResponse>, AppError> {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };
        Ok(Json(response_json))
    }
}

impl Feature for FeatureExample {
    fn create_router() -> Router {
        Router::new().route("/api/health_check", get(FeatureExample::health_check))
    }

    fn create_swagger() -> SwaggerMeta {
        SwaggerMeta {
            key: String::from("/api/health_check"),
            value: PathItem::new(
                PathItemType::Get,
                OperationBuilder::new()
                    .responses(
                        utoipa::openapi::ResponsesBuilder::new()
                            .response(
                                "200",
                                utoipa::openapi::ResponseBuilder::new()
                                    .description("Pet found succesfully")
                                    .content(
                                        "application/json",
                                        utoipa::openapi::Content::new(
                                            utoipa::openapi::Ref::from_response_name(
                                                "GenericResponse",
                                            ),
                                        ),
                                    ),
                            )
                            .response("404", utoipa::openapi::Response::new("Pet was not found")),
                    )
                    .operation_id(Some("get_pet_by_id"))
                    .deprecated(Some(utoipa::openapi::Deprecated::False))
                    .summary(Some("Get pet by id"))
                    .description(Some(
                        "Get pet by id\n\nGet pet from database by pet database id\n",
                    ))
                    .tag("health_check_api"),
            ),
        }
    }
}
