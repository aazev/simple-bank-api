use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    decrypt_user_key, load_master_key, structs::encrypted_field::EncryptedField,
    traits::encryptable::Encryptable,
};

use super::user_dto::User;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub user_id: Uuid, // Owner of the account
    pub balance: EncryptedField<f64>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl Account {
    pub fn new(user: &User, balance: f64) -> Result<Self, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(Self {
            id: Uuid::now_v7(),
            user_id: user.id,
            balance: balance.encrypt(&key)?,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
        })
    }

    pub fn get_balance(&self, user: &User) -> Result<f64, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(f64::decrypt(&self.balance, &key)?)
    }

    pub fn update_balance(&mut self, user: &User, new_balance: f64) -> Result<(), anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        self.balance = new_balance.encrypt(&key)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountCreate {
    pub user_id: Uuid,
    pub balance: f64,
}

impl AccountCreate {
    pub fn to_account(&self, user: &User) -> Result<Account, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(Account {
            id: Uuid::now_v7(),
            user_id: self.user_id,
            balance: self.balance.encrypt(&key)?,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: None,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct AccountModel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: f64,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

impl AccountModel {
    pub fn from_dto(account: &Account, user: &User) -> Result<Self, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(Self {
            id: account.id,
            user_id: account.user_id,
            balance: f64::decrypt(&account.balance, &key)?,
            created_at: account.created_at,
            updated_at: account.updated_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation_and_balance() {
        let user = User::new(
            "test".to_string(),
            "test".to_string(),
            Some(true),
            Some("test".to_string()),
        )
        .expect("User creation failed");
        let initial_balance = 1000.0;

        let account = Account::new(&user, initial_balance).expect("Account creation failed");

        let balance = account.get_balance(&user).expect("Failed to get balance");

        assert_eq!(initial_balance, balance);
    }

    #[test]
    fn test_account_balance_with_wrong_key_fails() {
        let user = User::new(
            "test".to_string(),
            "test".to_string(),
            Some(true),
            Some("test".to_string()),
        )
        .expect("User creation failed");
        let wrong_user = User::new(
            "wrong".to_string(),
            "wrong".to_string(),
            Some(true),
            Some("wrong".to_string()),
        )
        .expect("User creation failed");
        let initial_balance = 1000.0;

        let account = Account::new(&user, initial_balance).expect("Account creation failed");

        let result = account.get_balance(&wrong_user);

        assert!(
            result.is_err(),
            "Retrieving balance with wrong key should fail"
        );
    }
}
