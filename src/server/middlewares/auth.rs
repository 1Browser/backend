use crate::database::models::user::User;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub exp: usize,
}

pub async fn authorize(mut request: Request, next: Next) -> Response {
    let header = match request.headers().get(http::header::AUTHORIZATION) {
        None => return StatusCode::UNAUTHORIZED.into_response(),
        Some(header) => match header.to_str() {
            Ok(header) => header,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        },
    };

    let token = match header.split_whitespace().nth(1) {
        Some(token) => token,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let token_data = {
        let decoding_key = DecodingKey::from_secret("1browser".as_bytes());

        match jsonwebtoken::decode::<Claims>(token, &decoding_key, &Validation::default()) {
            Ok(data) => data,
            Err(error) => {
                error!(?error);

                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
    };

    let pg_pool = match request.extensions().get::<PgPool>() {
        Some(pool) => pool,
        None => {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let user = match User::get(pg_pool, token_data.claims.user_id).await {
        Ok(user) => user,
        Err(error) => {
            error!(?error);

            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    request.extensions_mut().insert(user);

    next.run(request).await
}
