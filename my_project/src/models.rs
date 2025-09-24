// src/models.rs

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Added `Clone` to the list of derived traits.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User ID (UUID)
    pub email: String,
    pub exp: usize,
}

// Data structure for a user in the database.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
}

// Data structure for user data sent to the frontend.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub api_usage_count: i64,
}

// Payload for updating a user.
#[derive(Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub password: Option<String>,
}

// Payload for the /register endpoint.
#[derive(Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
}

// Payload for the /login endpoint.
#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

// The response from a successful login.
#[derive(Serialize)]
pub struct Token {
    pub token: String,
}

// Data structure for an API usage record.
#[derive(Serialize)]
pub struct ApiUsage {
    pub id: i32,
    pub user_id: String,
    pub api_link: String,
    pub timestamp: String,
}