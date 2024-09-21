use crate::database::Database;
use ::oauth2::basic::BasicClient;
use axum::{Extension, Router};

mod oauth2;

pub fn new(database: Database, oauth2_client: BasicClient) -> Router {
    Router::new()
        .nest("/oauth2", oauth2::new_router(oauth2_client))
        .layer(Extension(database))
}
