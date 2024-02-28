use axum::{routing::get, Json};
use utoipa::openapi::{path::OperationBuilder, PathItem, PathItemType};

use crate::web_core::{dtos::GenericResponse, features::Feature, types::AppResult};

pub struct FeatureSample;

impl FeatureSample {
    pub async fn health_check() -> AppResult<Json<GenericResponse>> {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };
        Ok(Json(response_json))
    }
}

impl Feature for FeatureSample {
    fn new() -> Self {
        FeatureSample{}
    }

    fn add_routers(
        router: axum::Router<crate::web_core::routes::AppState>,
    ) -> axum::Router<crate::web_core::routes::AppState> {
        router.route("/api/test", get(FeatureSample::health_check))
    }

    fn add_swagger(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.paths.paths.insert(
            String::from("/api/test"),
            PathItem::new(
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
                    .tag("pet_api"),
            ),
        );
    }
}
