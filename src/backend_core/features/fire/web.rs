use std::collections::HashSet;
use std::sync::Arc;

use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::transform::TransformOperation;
use axum::extract::State;
use axum::{async_trait, http::StatusCode};
use futures::TryStreamExt;
use mongodb::bson::Document;
use mongodb::error::Error as MongoError;
use mongodb::Collection;
use schemars::JsonSchema;
use serde::Serialize;
use tokio::sync::mpsc::{Receiver, Sender};

use super::notifications::{FireIotNotification, FireWebNotification};
use crate::backend_core::features::WebFeature;
use crate::backend_core::utils::non_primitive_cast;
use crate::json::Json;

#[derive(Serialize, JsonSchema)]
pub struct FireResponse {
    pub status: String,
    pub message: String,
}

struct FireAppState {
    mongoc: mongodb::Client,
}

pub struct WebFireFeature {
    mongoc: mongodb::Client,
    iot_tx: Sender<FireWebNotification>,
    iot_rx: Receiver<FireIotNotification>,
}

impl WebFireFeature {
    async fn messages(state: State<Arc<FireAppState>>) -> impl IntoApiResponse {
        let collection = state
            .mongoc
            .default_database()
            .unwrap()
            .collection("fireMessages");

        // Assuming you have a function to retrieve all records from the collection
        match retrieve_all_records(&collection).await {
            Ok(records) => {
                // Extract messages from records and combine them
                let combined_message: String = records
                    .iter()
                    .filter_map(|record| record.get_str("message").ok())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(", ");

                (
                    StatusCode::OK,
                    Json(FireResponse {
                        status: "success".to_string(),
                        message: format!(
                            "Retrieved records successfully. Messages: {}",
                            combined_message
                        ),
                    }),
                )
            }
            Err(err) => {
                eprintln!("Failed to retrieve records: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(FireResponse {
                        status: "error".to_string(),
                        message: "Failed to retrieve records".into(),
                    }),
                )
            }
        }
    }

    pub fn messages_docs(op: TransformOperation) -> TransformOperation {
        op.description("Retrieve all fire metric messages")
            .tag("FIRE")
            .response::<200, Json<FireResponse>>()
    }
}

#[async_trait]
impl WebFeature for WebFireFeature {
    fn create<W: 'static, I: 'static>(
        mongoc: mongodb::Client,
        iot_tx: Sender<W>,
        iot_rx: Receiver<I>,
    ) -> Option<Self> {
        Some(WebFireFeature {
            mongoc,
            iot_tx: non_primitive_cast(iot_tx)?,
            iot_rx: non_primitive_cast(iot_rx)?,
        })
    }

    fn name() -> String
    where
        Self: Sized,
    {
        "fire".into()
    }

    fn get_module_name(&self) -> String {
        "fire".into()
    }

    fn create_router(&mut self) -> ApiRouter {
        let app_state = Arc::new(FireAppState {
            mongoc: self.mongoc.clone(),
        });

        ApiRouter::new().api_route(
            "/messages",
            get_with(WebFireFeature::messages, WebFireFeature::messages_docs).with_state(app_state),
        )
    }

    async fn run_loop(&mut self) {}
}

async fn retrieve_all_records(
    collection: &Collection<Document>,
) -> Result<Vec<Document>, MongoError> {
    let mut cursor = collection.find(None, None).await?;
    let mut records = Vec::new();

    while let Some(document) = cursor.try_next().await? {
        records.push(document)
    }

    Ok(records)
}
