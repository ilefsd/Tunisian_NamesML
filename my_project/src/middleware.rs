// src/middleware.rs
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

// Import the shared models
use crate::{db::ConnectionPool, models::Claims};

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

            // Use the correct, unified Claims struct for decoding
            if let Ok(token_data) = decode::<Claims>(token, &decoding_key, &validation) {
                // BUG FIX: The 'sub' claim IS the user_id.
                // We can use it directly without another database query.
                let user_id = token_data.claims.sub;
                let api_link = request.uri().to_string();

                let conn = pool.get().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

                conn.execute(
                    "INSERT INTO api_usage (user_id, api_link) VALUES ($1, $2)",
                    &[&user_id, &api_link],
                )
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to track API usage: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR
                    })?;
            }
        }
    }

    Ok(next.run(request).await)
}

pub async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let decoding_key = DecodingKey::from_secret("secret".as_ref());
    let validation = Validation::default();

    // Also use the unified Claims struct here
    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
        eprintln!("Auth error: {:?}", e);
        StatusCode::UNAUTHORIZED
    })?;

    // Optional: Pass claims to handlers via request extensions
    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}