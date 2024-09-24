use crate::database::models::user::User;
use crate::server::middlewares::auth::Claims;
use axum::extract::Query;
use axum::response::{IntoResponse, Redirect, Response};
use axum::response::Result;
use axum::routing::get;
use axum::{Extension, Json, Router};
use http::StatusCode;
use jsonwebtoken::{EncodingKey, Header};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tracing::error;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        authorize,
        callback,
    ),
    components(schemas(User)),
)]
pub(super) struct Oauth2Api;

pub fn new_router(oauth2_client: BasicClient) -> Router {
    Router::new()
        .route("/authorize", get(authorize))
        .route("/callback", get(callback))
        .layer(Extension(oauth2_client))
}

/// Authorize with Discord
/// 
/// Redirect to the Discord authorize page.
#[utoipa::path(
    get,
    path = "/authorize",
    responses(
        (status = 302, description = "Redirect to the Discord authorize page"),
    )
)]
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

/// Callback from Discord
/// 
/// Callback from Discord after the user has authorized the application.
#[utoipa::path(
    get,
    path = "/callback",
    responses(
        (status = 302, description = "Redirect to the application"),
    )
)]
async fn callback(
    Query(callback): Query<Callback>,
    Extension(oauth2_client): Extension<BasicClient>,
    Extension(pg_pool): Extension<PgPool>,
) -> Result<Response, StatusCode> {
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

    let user = match &user.email {
        None => {
            let body = json!({
                "error": "The discord account doesn't have an email address linked to it."
            });

            return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response());
        }
        Some(email) => {
            let avatar = match user.avatar {
                None => {
                    let hash = {
                        let mut hasher = Sha256::new();
                        hasher.update(email.as_bytes());

                        format!("{:x}", hasher.finalize())
                    };

                    format!("https://gravatar.com/avatar/{}?d=retro", hash)
                }
                Some(image_hash) => format!("https://cdn.discordapp.com/avatars/{}/{}.png", user.id, image_hash)
            };

            dbg!(&avatar);

            User::register_or_login(pg_pool, email, &avatar)
                .await
                .inspect_err(|error| error!(?error))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    let claims = Claims {
        user_id: user.id,
        exp: u32::MAX as usize,
    };

    // TODO Replace the secret.
    let jwt_encoding_key = EncodingKey::from_secret("1browser".as_bytes());

    let token = jsonwebtoken::encode(&Header::default(), &claims, &jwt_encoding_key)
        .inspect_err(|error| error!(?error))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Redirect::temporary(&format!("https://app.1browser.one/signin/discord?token={}", token)).into_response())
}
