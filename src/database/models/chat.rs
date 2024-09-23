use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Chat {
    pub user_id: Uuid,
    pub article_url: String,
    pub article_content: String,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl Chat {
    pub async fn save(pg_pool: &PgPool, user_id: Uuid, article_url: &str, article_content: &str, message: &str) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO chat (user_id, article_url, article_content, message) VALUES ($1, $2, $3, $4) ON CONFLICT (user_id, article_url) DO UPDATE SET message = EXCLUDED.message;"
        )
            .bind(user_id)
            .bind(article_url)
            .bind(article_content)
            .bind(message)
            .execute(pg_pool)
            .await?;

        Ok(())
    }

    pub async fn get(pg_pool: &PgPool, user_id: &Uuid, article_url: &str) -> sqlx::Result<Self> {
        let chat: Chat = sqlx::query_as(
            "SELECT * FROM chat WHERE user_id = $1 AND article_url = $2"
        )
            .bind(user_id)
            .bind(article_url)
            .fetch_one(pg_pool)
            .await?;

        Ok(chat)
    }
}
