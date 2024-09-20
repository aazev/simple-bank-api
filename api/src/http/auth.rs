use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub created_at: NaiveDateTime,
}

impl AuthResponse {
    pub fn new(token: String) -> Self {
        Self {
            token,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}
