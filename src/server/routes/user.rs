use crate::database::models::user::User;
use crate::server::middlewares::auth::authorize;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};

pub fn new_router() -> Router {
    Router::new()
        .route("/@me", get(get_me))
        .layer(middleware::from_fn(authorize))
}

async fn get_me(Extension(user): Extension<User>) -> Json<User> {
    Json(user)
}
