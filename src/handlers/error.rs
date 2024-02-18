use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Mongodb error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("Error during mongodb query: {0}")]
    MongoQueryError(mongodb::error::Error),
    #[error("Dulicate key error occurred: {0}")]
    MongoDuplicateError(mongodb::error::Error),
    #[error("Could not serialize data: {0}")]
    MongoSerializeBsonError(bson::ser::Error),
    #[error("Could not deserialize bson: {0}")]
    MongoDeserializeBsonError(bson::de::Error),
    #[error("Could not access field in document: {0}")]
    MongoDataError(#[from] bson::document::ValueAccessError),
    #[error(transparent)]
    AxumError(#[from] axum::Error),
    #[error("Invalid id used: {0}")]
    InvalidIDError(String),
    #[error("{0}")]
    BadRequest(#[from] BadRequest),
    #[error("{0}")]
    NotFound(#[from] NotFound),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    AddrParseError(#[from] std::net::AddrParseError),
    #[error(transparent)]
    UnknownError(#[from] anyhow::Error),
}

#[derive(Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(Error, Debug)]
#[error("Not found")]
pub struct NotFound {}

impl AppError {
    fn get_codes(&self) -> (StatusCode, &str, &str) {
        match self {
            AppError::MongoError(e) => {
                eprintln!("MongoDB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "failed", "MongoDB error")
            }
            AppError::MongoQueryError(e) => {
                eprintln!("Error during mongodb query: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error during mongodb query",
                )
            }
            AppError::MongoDuplicateError(e) => {
                eprintln!("MongoDB error: {:?}", e);
                (StatusCode::CONFLICT, "failed", "Duplicate key error")
            }
            AppError::MongoSerializeBsonError(e) => {
                eprintln!("Error seserializing BSON: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error seserializing BSON",
                )
            }
            AppError::MongoDeserializeBsonError(e) => {
                eprintln!("Error deserializing BSON: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error deserializing BSON",
                )
            }
            AppError::MongoDataError(e) => {
                eprintln!("validation error: {:?}", e);
                (StatusCode::BAD_REQUEST, "failed", "Validation error")
            }
            AppError::InvalidIDError(e) => {
                eprintln!("Invalid ID: {:?}", e);
                (StatusCode::BAD_REQUEST, "failed", e.as_str())
            }
            AppError::AxumError(e) => {
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "failed", "Axum Error")
            }
            AppError::BadRequest(e) => {
                eprintln!("{:?}", e);
                (StatusCode::BAD_REQUEST, "failed", "Invalid Body")
            }
            AppError::NotFound(e) => {
                eprintln!("{:?}", e);
                (
                    StatusCode::NOT_FOUND,
                    "failed",
                    "Route does not exist on the server",
                )
            }
            AppError::IoError(e) => {
                eprintln!("{:?}", e);
                match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        (StatusCode::NOT_FOUND, "failed", "Not found error")
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        (StatusCode::FORBIDDEN, "failed", "Forbidden error")
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, "failed", "IO error"),
                }
            }
            AppError::AddrParseError(e) => {
                eprintln!("{:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Address parser error",
                )
            }
            AppError::UnknownError(e) => {
                eprintln!("{:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "error", "Unknown error")
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, status, message) = self.get_codes();
        let body = Json(json!({ "status": status, "message": message }));

        (status_code, body).into_response()
    }
}
