use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub avatar: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub async fn register_or_login(pg_pool: PgPool, email: &str, avatar: &str) -> sqlx::Result<Self> {
        let user: User = sqlx::query_as(
            "INSERT INTO users (email, avatar) VALUES ($1, $2) ON CONFLICT (email) DO UPDATE SET email = EXCLUDED.email RETURNING *;"
        )
            .bind(email)
            .bind(avatar)
            .fetch_one(&pg_pool)
            .await?;

        Ok(user)
    }

    pub async fn get(pg_pool: &PgPool, id: Uuid) -> sqlx::Result<Self> {
        let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(pg_pool)
            .await?;

        Ok(user)
    }
}
