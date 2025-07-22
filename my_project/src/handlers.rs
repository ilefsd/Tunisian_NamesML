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
