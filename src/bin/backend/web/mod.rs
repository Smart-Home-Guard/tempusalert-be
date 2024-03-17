mod auth_apis;
mod push_api;
mod register_api;
mod utils;
mod middlewares;
mod doc;

use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{OpenApi, Tag},
    transform::TransformOpenApi,
};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;

use crate::{config::WebConfig, AppResult};
use axum::{http::{StatusCode, Uri}, Extension, Json};
use tempusalert_be::backend_core::features::WebFeature;

use self::doc::docs_routes;

pub struct WebTask {
    pub config: WebConfig,
    tcp: tokio::net::TcpListener,
    features: Vec<Arc<Mutex<dyn WebFeature + Send + Sync>>>,
    router: ApiRouter,
}

impl WebTask {
    pub async fn create(
        mut config: WebConfig,
        features: Vec<Arc<Mutex<dyn WebFeature + Send + Sync>>>,
    ) -> AppResult<Self> {
        let tcp = tokio::net::TcpListener::bind(config.get_socket_addr()?).await?;
        let addr = tcp.local_addr()?;
        config.port = addr.port();
        let router = ApiRouter::new();
        Ok(Self {
            config,
            tcp,
            features,
            router,
        })
    }

    pub async fn run(mut self) -> AppResult {
        tracing_subscriber::fmt::init();

        aide::gen::on_error(|error| {
            println!("{error}");
        });

        aide::gen::extract_schemas(true);

        let mut api = OpenApi::default();

        // authentication routes
        self.router = self
            .router
            .fallback(|uri: Uri| async move {
                (StatusCode::NOT_FOUND, Json(format!("No route for {uri}")))
            })
            .nest_api_service("/auth/iot", auth_apis::iot_auth_routes())
            .nest_api_service("/auth/web", auth_apis::web_auth_routes())
            .nest_api_service("/auth/register", register_api::register_routes())
            .nest_api_service("/push/credential", push_api::push_routes());

        for feat in &mut self.features {
            self.router = self.router.nest_api_service(
                format!("/api/{}", feat.clone().lock().await.get_module_name()).as_str(),
                feat.lock().await.create_router(),
            )
        }

        let router = self
            .router
            .nest_api_service("/docs", docs_routes())
            .finish_api_with(&mut api, WebTask::api_docs)
            .layer(Extension(Arc::new(api)))
            .layer(TraceLayer::new_for_http())
            .into_make_service();
        tokio::spawn(async move {
            println!(
                "Web server ready to server on {}://{}:{}",
                self.config.protocol, self.config.addr, self.config.port
            );
            println!(
                "Check web server doc at {}://{}:{}/docs",
                self.config.protocol, self.config.addr, self.config.port
            );
            axum::serve(self.tcp, router).await
        });
        let mut join_handles = vec![];
        for feat in self.features {
            join_handles.push(tokio::spawn(async move {
                loop {
                    let mut feat = feat.lock().await;
                    feat.process_next_iot_push_message().await;
                }
            }));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Tempusalert Open API")
            .summary("Crates for server apps in the backend: the IoT server, the MQTT broker and the Web server")
            .tag(Tag {
                name: "tempusalert".into(),
                description: Some("Smart Home Guard".into()),
                ..Default::default()
            })
            .security_scheme(
                "ApiKey",
                aide::openapi::SecurityScheme::ApiKey {
                    location: aide::openapi::ApiKeyLocation::Header,
                    name: "X-Auth-Key".into(),
                    description: Some("A key that is ignored.".into()),
                    extensions: Default::default(),
                },
            )
    }
}
