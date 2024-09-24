use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct Comment {
    pub id: Uuid,
    pub url: String,
    pub selector: String,
    pub origin: Option<String>,
    pub user_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Comment {
    pub async fn create(pg_pool: PgPool, url: &str, selector:&str, origin: Option<&str>, user_id: Uuid, content: &str) -> sqlx::Result<Self> {
        let comment: Comment = sqlx::query_as(
            "INSERT INTO comments (url, selector, origin, user_id, content) VALUES ($1, $2, $3, $4, $5) RETURNING *;"
        )
            .bind(url)
            .bind(selector)
            .bind(origin)
            .bind(user_id)
            .bind(content)
            .fetch_one(&pg_pool)
            .await?;

        Ok(comment)
    }

    pub async fn list(pg_pool: PgPool, url: &str, page: Option<i32>, limit: Option<i32>) -> sqlx::Result<Vec<Comment>> {
        let page = page.unwrap_or(0);
        let limit = limit.unwrap_or(100);

        let comments: Vec<Comment> = sqlx::query_as(
            "SELECT * FROM comments WHERE url = $1 ORDER BY created_at DESC OFFSET $2 LIMIT $3;"
        )
            .bind(url)
            .bind(page)
            .bind(limit)
            .fetch_all(&pg_pool)
            .await?;

        Ok(comments)
    }
}
