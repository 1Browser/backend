use ::oauth2::basic::BasicClient;
use axum::{Extension, Router};
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;

mod oauth2;
mod user;
mod summary;

pub fn new(pg_pool: PgPool, openai: OpenAI<OpenAIConfig>, oauth2_client: BasicClient) -> Router {
    Router::new()
        .nest("/oauth2", oauth2::new_router(oauth2_client))
        .nest("/users", user::new_router())
        .nest("/summary", summary::new_router(openai))
        .layer(CorsLayer::permissive())
        .layer(Extension(pg_pool))
}
