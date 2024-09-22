use crate::server::middlewares::auth::authorize;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};
use axum::extract::Path;
use http::StatusCode;
use sqlx::PgPool;
use tracing::error;
use axum::response::Result;
use uuid::Uuid;
use crate::database::models::user::User;

pub fn new_router() -> Router {
    Router::new()
        .route("/:id", get(get_user))
        .layer(middleware::from_fn(authorize))
}

async fn get_user(Path(id): Path<Uuid>, Extension(user): Extension<User>, Extension(pg_pool): Extension<PgPool>) -> Result<Json<User>, StatusCode> {
    if user.id != id {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pg_pool)
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user))
}
