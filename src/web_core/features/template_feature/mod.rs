use axum::{routing::get, Json};
use utoipa::openapi::{path::OperationBuilder, PathItem, PathItemType};
use crate::web_core::{dtos::GenericResponse, features::Feature, types::AppResult};

pub struct FeatureExample;

impl FeatureExample {
    pub async fn health_check() -> AppResult<Json<GenericResponse>> {
        const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

        let response_json = GenericResponse {
            status: "success".to_string(),
            message: MESSAGE.to_string(),
        };
        Ok(Json(response_json))
    }
}

impl Feature for FeatureExample {
    fn new() -> Self {
        FeatureExample{}
    }

    fn add_routers(
        router: axum::Router<crate::web_core::routes::AppState>,
    ) -> axum::Router<crate::web_core::routes::AppState> {
        router.route("/api/health_check", get(FeatureExample::health_check))
    }

    fn add_swagger(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.paths.paths.insert(
            String::from("/api/health_check"),
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
                    .tag("health_check_api"),
            ),
        );
    }
}
