use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
    #[schema(
        default = "vec![]",
        example = "['users', 'accounts', 'transactions', 'admin']"
    )]
    pub scopes: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
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
