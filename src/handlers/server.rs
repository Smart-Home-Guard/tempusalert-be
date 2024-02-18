use axum::Json;

use crate::dtos::GenericResponse;
use crate::types::AppResult;

// Health check.
#[utoipa::path(
    get,
    path = "/api/health_check",
    responses(
        (status = 200, description = "check service is up", body = [GenericResponse])
    )
)]
pub async fn health_check() -> AppResult<Json<GenericResponse>> {
    const MESSAGE: &str = "Build CRUD API with Rust and MongoDB";

    let response_json = GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };
    Ok(Json(response_json))
}

#[cfg(test)]
pub mod tests {

    use super::*;

    #[tokio::test]
    async fn test_health_check_handler() {
        assert_eq!(
            health_check().await.unwrap().message,
            "Build CRUD API with Rust and MongoDB"
        );
    }
}
