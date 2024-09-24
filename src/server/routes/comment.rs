use axum::{extract::Query, middleware, response:: Result, routing::{get, post}, Extension, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{database::models::{comment::Comment, user::User}, server::middlewares::{auth::{authorize}}};

pub fn new_router() -> Router {
    Router::new()
        .route("/", get(list_comments))
        .route("/", post(create_comment).layer(middleware::from_fn(authorize)))
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    url: String,
    selector: String,
    origin: Option<String>,
    content: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCommentResponse {
    id: Uuid,
}

pub async fn create_comment(
    Extension(user): Extension<User>,
    Extension(pg_pool): Extension<PgPool>,
    Json(request): Json<CreateCommentRequest>,
) -> Result<Json<CreateCommentResponse>, StatusCode> {
    let comment = Comment::create(pg_pool, &request.url, &request.selector, request.origin.as_deref(), user.id,&request.content)
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CreateCommentResponse{ id: comment.id }))
}

#[derive(Deserialize)]
pub struct ListCommentsQuery {
    url: String,
    page: Option<i32>,
    limit: Option<i32>,
}

pub type ListCommentResponse = Vec<Comment>;

pub async fn list_comments(
    Extension(pg_pool): Extension<PgPool>,
    Query(query): Query<ListCommentsQuery,
>) -> Result<Json<ListCommentResponse>, StatusCode> {
    let comments = Comment::list(pg_pool, &query.url, query.page, query.limit)
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(comments))
}
