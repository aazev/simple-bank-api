use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{structs::encrypted_field::EncryptedField, traits::encryptable::Encryptable};

use super::user_dto::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub account_id: Uuid,
    pub user_id: Uuid, // Owner of the account
    pub balance: EncryptedField<f64>,
    pub created_at: NaiveDateTime,
}

impl Account {
    pub fn new(user: &User, balance: f64) -> Result<Self, anyhow::Error> {
        Ok(Self {
            account_id: Uuid::now_v7(),
            user_id: user.user_id,
            balance: balance.encrypt(&user.encryption_key),
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn get_balance(&self, user: &User) -> Result<f64, anyhow::Error> {
        Ok(f64::decrypt(&self.balance, &user.encryption_key))
    }
}
