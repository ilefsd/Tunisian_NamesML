use axum::{
    extract::{State, Request},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use crate::db::ConnectionPool;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn track_api_usage(
    State(pool): State<ConnectionPool>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            let decoding_key = DecodingKey::from_secret("secret".as_ref());
            let validation = Validation::default();
            if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
                let user_email = token_data.claims.sub;
                let api_link = request.uri().to_string();

                let conn = pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let user_row = conn.query_one("SELECT id FROM users WHERE email = $1", &[&user_email]).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                let user_id: String = user_row.get("id");

                conn.execute(
                    "INSERT INTO api_usage (user_id, api_link) VALUES ($1, $2)",
                    &[&user_id, &api_link],
                )
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    if let Some(auth_header) = auth_header {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            let decoding_key = DecodingKey::from_secret("secret".as_ref());
            let validation = Validation::default();
            if decode::<Claims>(token, &decoding_key, &validation).is_ok() {
                return Ok(next.run(request).await);
            }
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
