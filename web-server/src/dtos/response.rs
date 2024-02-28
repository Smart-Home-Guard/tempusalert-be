use serde::Serialize;
use utoipa::{ToResponse, ToSchema};

#[derive(Serialize, ToSchema, ToResponse)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
