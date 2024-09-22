use crate::server::Server;
use clap::Parser;
use langchain_rust::llm::{OpenAI, OpenAIConfig};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sqlx::PgPool;

mod database;
mod server;

#[derive(Parser)]
struct Args {
    #[arg(
        long = "database.url",
        default_value = "postgres://postgres@localhost:5432/1browser",
        env = "1BROWSER_DATABASE_URL"
    )]
    database_url: String,

    #[arg(long = "oauth2.client-id", env = "1BROWSER_OAUTH2_CLIENT_ID")]
    oauth2_client_id: String,
    #[arg(long = "oauth2.secret", env = "1BROWSER_OAUTH2_CLIENT_SECRET")]
    oauth2_client_secret: String,
    #[arg(long = "oauth2.redirect-url", env = "1BROWSER_OAUTH2_REDIRECT_URI")]
    oauth2_redirect_uri: String,

    #[arg(long = "openai.api-key", env = "1BROWSER_OPENAI_API_KEY")]
    openai_api_key: String,

    #[arg(long = "jwt.secret", env = "1BROWSER_JWT_SECRET")]
    jwt_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::try_parse()?;

    let pg_pool = {
        let pg_pool = PgPool::connect(&args.database_url).await?;

        sqlx::migrate!("src/database/migrations")
            .run(&pg_pool)
            .await?;

        pg_pool
    };


    let openai = {
        let openai_config = OpenAIConfig::default()
            .with_api_key(args.openai_api_key);

        OpenAI::new(openai_config)
    };

    let oauth2_client = BasicClient::new(
        ClientId::new(args.oauth2_client_id),
        Some(ClientSecret::new(args.oauth2_client_secret)),
        AuthUrl::new("https://discord.com/oauth2/authorize".to_string())?,
        Some(TokenUrl::new(
            "https://discord.com/api/oauth2/token".to_string(),
        )?),
    )
        .set_redirect_uri(RedirectUrl::new(args.oauth2_redirect_uri)?);

    Server::new(pg_pool, openai, oauth2_client).serve().await
}
