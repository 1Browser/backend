use ::oauth2::basic::BasicClient;
use axum::{Extension, Router};
use sqlx::PgPool;

mod oauth2;
mod user;

pub fn new(pg_pool: PgPool, oauth2_client: BasicClient) -> Router {
    Router::new()
        .nest("/oauth2", oauth2::new_router(oauth2_client))
        .nest("/users", user::new_router())
        .layer(Extension(pg_pool))
}
