use anyhow::anyhow;
use axum::Router;
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use oauth2::basic::BasicClient;
use sqlx::PgPool;
use tokio::net::TcpListener;

mod routes;
pub mod middlewares;

pub struct Server {
    router: Router,
}

impl Server {
    pub fn new(pg_pool: PgPool, openai: OpenAI<OpenAIConfig>, oauth2_client: BasicClient) -> Self {
        Self {
            router: routes::new(pg_pool, openai, oauth2_client),
        }
    }

    pub async fn serve(self) -> anyhow::Result<()> {
        let tcp_listener = TcpListener::bind("0.0.0.0:80").await?;

        axum::serve(tcp_listener, self.router)
            .await
            .map_err(|error| anyhow!(error))
    }
}
