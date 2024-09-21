use crate::database::Database;
use axum::extract::Query;
use axum::response::Redirect;
use axum::response::Result;
use axum::routing::get;
use axum::{Extension, Router};
use http::StatusCode;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};
use serde::Deserialize;
use tracing::error;

pub fn new_router(oauth2_client: BasicClient) -> Router {
    Router::new()
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
        .layer(Extension(oauth2_client))
}

async fn authorize(Extension(oauth2_client): Extension<BasicClient>) -> Redirect {
    let url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}&scope=identify+email",
        **oauth2_client.client_id(),
        **oauth2_client.redirect_url().unwrap(),
    );

    Redirect::temporary(&url)
}

#[derive(Deserialize)]
struct Callback {
    code: String,
}

async fn callback(
    Query(callback): Query<Callback>,
    Extension(oauth2_client): Extension<BasicClient>,
    Extension(_): Extension<Database>,
) -> Result<(), StatusCode> {
    let token = oauth2_client
        .exchange_code(AuthorizationCode::new(callback.code))
        .request_async(async_http_client)
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let token = format!("Bearer {}", token.access_token().secret());

    let user = serenity::http::Http::new(&token)
        .get_current_user()
        .await
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    dbg!(user);

    Ok(())
}
