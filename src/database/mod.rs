use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct Database {
    pg_pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, sqlx::Error> {
        let pg_pool = PgPool::connect(url).await?;

        Ok(Self { pg_pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("src/database/migrations")
            .run(&self.pg_pool)
            .await
    }
}
