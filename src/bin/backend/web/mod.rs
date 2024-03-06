use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::OpenApi,
};

use crate::{config::WebConfig, doc::docs_routes, AppResult};
use axum::Extension;
use tempusalert_be::backend_core::features::WebFeature;

pub struct WebTask {
    pub config: WebConfig,
    tcp: tokio::net::TcpListener,
    features: Vec<Box<dyn WebFeature + Send + Sync>>,
    router: ApiRouter,
}

impl WebTask {
    pub async fn create(
        mut config: WebConfig,
        features: Vec<Box<dyn WebFeature + Send + Sync>>,
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
                .nest_api_service("/", feat.create_router())
        }

        let router = self
            .router
            .nest_api_service("/docs", docs_routes())
            .finish_api(&mut api)
            .layer(Extension(Arc::new(api)));
        tokio::spawn(async move {
            println!("Web server ready to server on {}:{}", self.config.addr, self.config.port);
            println!("Check web server doc at {}:{}/docs", self.config.addr, self.config.port);
            axum::serve(self.tcp, router).await
        });
        let mut join_handles = vec![];
        for mut feat in self.features {
            join_handles.push(tokio::spawn(
                async move { feat.run_loop().await },
            ));
        }
        for handle in join_handles {
            handle.await.unwrap()
        }
        Ok(())
    }
}
