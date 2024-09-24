use axum::{extract::Query, middleware, response:: Result, routing::{get, post}, Extension, Json, Router};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use utoipa::{IntoParams, OpenApi, ToSchema};
use uuid::Uuid;

use crate::{database::models::{comment::Comment, user::User}, server::middlewares::auth::authorize};

#[derive(OpenApi)]
#[openapi(
    paths(create_comment, list_comments),
    components(schemas(
        Comment,
        CreateCommentRequest, CreateCommentResponse,
        ListCommentResponse,
    )),
)]
pub(super) struct CommentApi;

pub fn new_router() -> Router {
    Router::new()
        .route("/", get(list_comments))
        .route("/", post(create_comment).layer(middleware::from_fn(authorize)))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCommentRequest {
    url: String,
    selector: String,
    origin: Option<String>,
    content: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreateCommentResponse {
    id: Uuid,
}

/// Create a comment
///
/// Create a comment on a specific URL and selector.
#[utoipa::path(
    post,
    path = "",
    request_body = CreateCommentRequest,
    responses(
        (status = 200, description = "Comment created successfully", body = CreateCommentResponse),
    )
)]
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

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListCommentsQuery {
    url: String,
    page: Option<i32>,
    limit: Option<i32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListCommentResponse(Vec<Comment>);

/// List all comments
/// 
/// List all comments on a specific URL.
#[utoipa::path(
    get,
    path = "",
    params(ListCommentsQuery),
    responses(
        (status = 200, description = "List all comments successfully", body = ListCommentResponse)
    )
)]
pub async fn list_comments(
    Extension(pg_pool): Extension<PgPool>,
    Query(query): Query<ListCommentsQuery,
>) -> Result<Json<ListCommentResponse>, StatusCode> {
    let comments = Comment::list(pg_pool, &query.url, query.page, query.limit)
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ListCommentResponse(comments)))
}
