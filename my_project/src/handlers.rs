use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use jsonwebtoken::{encode, EncodingKey, Header};
use chrono::{Utc, Duration};

use crate::{
    db::ConnectionPool,
    models::{User,},
};

#[derive(Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct Token {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub async fn register(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<StatusCode, (StatusCode, String)> {
    let password_hash =
        hash(&payload.password, DEFAULT_COST).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?;

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        password_hash,
    };

    let conn = pool.get().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

    conn.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user.id.to_string(), &user.email, &user.password_hash],
    )
        .await
        .map_err(|e| {
            eprintln!("Failed to insert user: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user".to_string())
        })?;

    Ok(StatusCode::CREATED)
}

pub async fn login(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<Token>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

    let row = conn
        .query_one("SELECT * FROM users WHERE email = $1", &[&payload.email])
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let id_str: String = row.get("id");
    let user = User {
        id: Uuid::parse_str(&id_str).unwrap(),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
    };
    if !verify(&payload.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify password".to_string()))?
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let claims = Claims {
        sub: user.email,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret("secret".as_ref()))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token".to_string()))?;

    Ok(Json(Token { token }))
}

#[derive(Serialize)]
pub struct ApiUsage {
    pub id: i32,
    pub user_id: String,
    pub api_link: String,
    pub timestamp: String,
}

use std::time::SystemTime;

pub async fn get_api_usage(
    State(pool): State<ConnectionPool>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<ApiUsage>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

    let rows = conn
        .query("SELECT * FROM api_usage WHERE user_id = $1", &[&user_id])
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get API usage".to_string()))?;

    let mut api_usage = Vec::new();
    for row in rows {
        let timestamp: SystemTime = row.get("timestamp");
        let timestamp_dt: chrono::DateTime<Utc> = timestamp.into();
        api_usage.push(ApiUsage {
            id: row.get("id"),
            user_id: row.get("user_id"),
            api_link: row.get("api_link"),
            timestamp: timestamp_dt.to_rfc3339(),
        });
    }

    Ok(Json(api_usage))
}
