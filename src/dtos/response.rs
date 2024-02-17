use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
