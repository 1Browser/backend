use crate::database::models::user::User;
use crate::server::middlewares::auth::authorize;
use axum::routing::get;
use axum::{middleware, Extension, Json, Router};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(get_me),
    components(schemas(User)),
)]
pub(super) struct UserApi;

pub fn new_router() -> Router {
    Router::new()
        .route("/@me", get(get_me))
        .layer(middleware::from_fn(authorize))
}

/// Get current user
/// 
/// Get the current user profile.
#[utoipa::path(
    get,
    path = "/@me",
    responses(
        (status = 200, description = "User profile", body = User),
    )
)]
async fn get_me(Extension(user): Extension<User>) -> Json<User> {
    Json(user)
}
