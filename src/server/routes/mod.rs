use axum::routing::get;
use ::oauth2::basic::BasicClient;
use axum::{Extension, Json, Router};
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;

mod oauth2;
mod user;
mod summary;
mod comment;


pub fn new(pg_pool: PgPool, openai: OpenAI<OpenAIConfig>, oauth2_client: BasicClient) -> Router {
    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "1browser",
        ),
        servers(
            (url = "http://localhost/", description = "development"),
            (url = "https://1browser.fly.dev/", description = "production"),
        ),
        nest(
            (path = "/oauth2", api = oauth2::Oauth2Api),
            (path = "/users", api = user::UserApi),
            (path = "/comments", api = comment::CommentApi),
        )
    )]
    struct ApiDoc;

    Router::new()
        .route("openapi.json", get(|| async { Json(ApiDoc::openapi()) }))
        .nest("/oauth2", oauth2::new_router(oauth2_client))
        .nest("/users", user::new_router())
        .nest("/summary", summary::new_router(openai))
        .nest("/comments", comment::new_router())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(Extension(pg_pool))
}
