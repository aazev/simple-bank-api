use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use chrono::NaiveDateTime;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{encrypt_user_key, generate_random_key, load_master_key};

#[derive(Debug, Serialize, Deserialize, FromRow, Default, Clone)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub active: bool,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub encryption_key: Vec<u8>, // User-specific encryption key
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl User {
    pub fn new(
        name: String,
        email: String,
        active: Option<bool>,
        password: Option<String>,
    ) -> anyhow::Result<Self> {
        // use argon2 to hash the password
        let salt = SaltString::generate(&mut OsRng);
        let password = Argon2::default()
            .hash_password(password.unwrap().as_bytes(), &salt)?
            .to_string();

        let user_key = generate_random_key();
        let master_key = load_master_key()?;
        let encryption_key = encrypt_user_key(&user_key, &master_key)?;

        Ok(Self {
            id: Uuid::now_v7(),
            name,
            email,
            active: active.unwrap_or(true),
            password,
            encryption_key,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
    pub active: Option<bool>,
    pub password: Option<String>,
}

impl TryFrom<UserCreate> for User {
    type Error = anyhow::Error;

    fn try_from(user_create: UserCreate) -> anyhow::Result<Self> {
        Self::new(
            user_create.name,
            user_create.email,
            user_create.active,
            user_create.password,
        )
    }
}

impl TryFrom<&UserCreate> for User {
    type Error = anyhow::Error;

    fn try_from(user_create: &UserCreate) -> anyhow::Result<Self> {
        Self::new(
            user_create.name.clone(),
            user_create.email.clone(),
            user_create.active,
            user_create.password.clone(),
        )
    }
}
