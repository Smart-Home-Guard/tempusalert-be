use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("mongodb error: {0}")]
    MongoError(#[from] mongodb::error::Error),
    #[error("error during mongodb query: {0}")]
    MongoQueryError(mongodb::error::Error),
    #[error("dulicate key error occurred: {0}")]
    MongoDuplicateError(mongodb::error::Error),
    #[error("could not serialize data: {0}")]
    MongoSerializeBsonError(bson::ser::Error),
    #[error("could not deserialize bson: {0}")]
    MongoDeserializeBsonError(bson::de::Error),
    #[error("could not access field in document: {0}")]
    MongoDataError(#[from] bson::document::ValueAccessError),
    #[error("invalid id used: {0}")]
    InvalidIDError(String),
    #[error("{0}")]
    BadRequest(#[from] BadRequest),
    #[error("{0}")]
    NotFound(#[from] NotFound),
}

#[derive(Error, Debug)]
#[error("Bad Request")]
pub struct BadRequest {}

#[derive(Error, Debug)]
#[error("Not found")]
pub struct NotFound {}

impl Error {
    fn get_codes(&self) -> (StatusCode, &str, &str) {
        match self {
            Error::MongoError(e) => {
                eprintln!("MongoDB error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "failed", "MongoDB error")
            }
            Error::MongoQueryError(e) => {
                eprintln!("Error during mongodb query: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error during mongodb query",
                )
            }
            Error::MongoDuplicateError(e) => {
                eprintln!("MongoDB error: {:?}", e);
                (StatusCode::CONFLICT, "failed", "Duplicate key error")
            }
            Error::MongoSerializeBsonError(e) => {
                eprintln!("Error seserializing BSON: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error seserializing BSON",
                )
            }
            Error::MongoDeserializeBsonError(e) => {
                eprintln!("Error deserializing BSON: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed",
                    "Error deserializing BSON",
                )
            }
            Error::MongoDataError(e) => {
                eprintln!("validation error: {:?}", e);
                (StatusCode::BAD_REQUEST, "failed", "validation error")
            }
            Error::InvalidIDError(e) => {
                eprintln!("Invalid ID: {:?}", e);
                (StatusCode::BAD_REQUEST, "failed", e.as_str())
            }
            Error::BadRequest(e) => {
                eprintln!("{:?}", e);
                (StatusCode::BAD_REQUEST, "failed", "Invalid Body")
            }
            Error::NotFound(e) => {
                eprintln!("{:?}", e);
                (
                    StatusCode::NOT_FOUND,
                    "failed",
                    "Route does not exist on the server",
                )
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, status, message) = self.get_codes();
        let body = Json(json!({ "status": status, "message": message }));

        (status_code, body).into_response()
    }
}