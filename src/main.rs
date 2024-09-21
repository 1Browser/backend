use crate::database::Database;
use crate::server::Server;
use clap::Parser;
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

mod database;
mod server;

#[derive(Parser)]
struct Args {
    #[arg(
        long = "database.url",
        default_value = "postgres://postgres@localhost:5432/1browser"
    )]
    database_url: String,

    #[arg(long = "oauth2.client-id")]
    oauth2_client_id: String,
    #[arg(long = "oauth2.secret")]
    oauth2_client_secret: String,
    #[arg(long = "oauth2.redirect-url")]
    oauth2_redirect_uri: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let args = Args::try_parse()?;

    let database = {
        let database = Database::connect(&args.database_url).await?;
        database.migrate().await?;

        database
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

    Server::new(database, oauth2_client).serve().await
}
