// src/handlers.rs

use axum::{extract::State, http::StatusCode, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use std::time::SystemTime;
use uuid::Uuid;

use crate::{
    db::ConnectionPool,
    models::{ApiUsage, Claims, LoginUser, RegisterUser, Token, User}, // Import all models
};

pub async fn register(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<StatusCode, (StatusCode, String)> {
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?;

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        password_hash,
    };

    let conn = pool
        .get()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

    conn.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user.id, &user.email, &user.password_hash],
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
    let conn = pool
        .get()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

    let row = conn
        .query_one("SELECT * FROM users WHERE email = $1", &[&payload.email])
        .await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    let user = User {
        id: row.get("id"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
    };
    if !verify(&payload.password, &user.password_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to verify password".to_string()))?
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    // This now uses the correct, shared Claims struct
    let claims = Claims {
        sub: user.id.to_string(), // Convert Uuid to String here
        email: user.email,
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("secret".as_ref()),
    )
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create token".to_string()))?;

    Ok(Json(Token { token }))
}

pub async fn get_api_usage(
    State(pool): State<ConnectionPool>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<Json<Vec<ApiUsage>>, (StatusCode, String)> {
    let conn = pool
        .get()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get connection".to_string()))?;

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

pub async fn get_users(
    State(pool): State<ConnectionPool>,
) -> Result<Json<Vec<crate::models::UserResponse>>, (StatusCode, String)> {
    let conn = pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get connection".to_string(),
        )
    })?;

    let rows = conn
        .query("SELECT id, email FROM users", &[])
        .await
        .map_err(|e| {
            eprintln!("Failed to get users: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get users".to_string(),
            )
        })?;

    let users = rows
        .into_iter()
        .map(|row| {
            let id: Uuid = row.get(0);
            crate::models::UserResponse {
                id: id.to_string(),
                email: row.get(1),
            }
        })
        .collect();

    Ok(Json(users))
}

pub async fn create_user(
    State(pool): State<ConnectionPool>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<crate::models::UserResponse>, (StatusCode, String)> {
    let password_hash = hash(&payload.password, DEFAULT_COST).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to hash password".to_string(),
        )
    })?;

    let user = User {
        id: Uuid::new_v4(),
        email: payload.email,
        password_hash,
    };

    let conn = pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get connection".to_string(),
        )
    })?;

    conn.execute(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3)",
        &[&user.id, &user.email, &user.password_hash],
    )
        .await
        .map_err(|e| {
            eprintln!("Failed to insert user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to register user".to_string(),
            )
        })?;

    Ok(Json(crate::models::UserResponse {
        id: user.id.to_string(),
        email: user.email,
    }))
}

pub async fn update_user(
    State(pool): State<ConnectionPool>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
    Json(payload): Json<crate::models::UpdateUser>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&user_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid user ID".to_string(),
        )
    })?;

    let conn = pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get connection".to_string(),
        )
    })?;

    if let Some(email) = payload.email {
        conn.execute("UPDATE users SET email = $1 WHERE id = $2", &[&email, &user_id])
            .await
            .map_err(|e| {
                eprintln!("Failed to update user email: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to update user".to_string(),
                )
            })?;
    }

    if let Some(password) = payload.password {
        let password_hash = hash(&password, DEFAULT_COST).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string(),
            )
        })?;
        conn.execute(
            "UPDATE users SET password_hash = $1 WHERE id = $2",
            &[&password_hash, &user_id],
        )
            .await
            .map_err(|e| {
                eprintln!("Failed to update user password: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to update user".to_string(),
                )
            })?;
    }

    Ok(StatusCode::OK)
}

pub async fn delete_user(
    State(pool): State<ConnectionPool>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&user_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "Invalid user ID".to_string(),
        )
    })?;

    let conn = pool.get().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to get connection".to_string(),
        )
    })?;

    conn.execute("DELETE FROM users WHERE id = $1", &[&user_id])
        .await
        .map_err(|e| {
            eprintln!("Failed to delete user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete user".to_string(),
            )
        })?;

    Ok(StatusCode::NO_CONTENT)
}