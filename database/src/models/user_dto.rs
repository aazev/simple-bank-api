use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub encryption_key: Vec<u8>, // User-specific encryption key
}

impl User {
    pub fn new(username: String, password_hash: String, encryption_key: Vec<u8>) -> Self {
        Self {
            user_id: Uuid::now_v7(),
            username,
            password_hash,
            encryption_key,
        }
    }
}
