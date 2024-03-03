use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{OpenApi, Tag},
    transform::TransformOpenApi,
};

use crate::{config::WebConfig, doc::docs_routes, AppResult};
use axum::Extension;
use tempusalert_be::backend_core::features::WebFeature;
use tokio::{join, sync::Mutex};

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
        aide::gen::on_error(|error| {
            println!("{error}");
        });

        aide::gen::extract_schemas(true);

        let mut api = OpenApi::default();

        for feat in &mut self.features {
            self.router = self
                .router
                .nest_api_service("/", feat.lock().await.create_router())
        }

        let router = self
            .router
            .nest_api_service("/docs", docs_routes())
            .finish_api(&mut api)
            .layer(Extension(Arc::new(api)));
        tokio::spawn(async { axum::serve(self.tcp, router).await });
        let mut join_handles = vec![];
        for feat in self.features {
            join_handles.push(tokio::spawn(
                async move { feat.lock().await.run_loop().await },
            ));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }
}
