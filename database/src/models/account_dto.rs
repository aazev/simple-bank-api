use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    decrypt_user_key, load_master_key, structs::encrypted_field::EncryptedField,
    traits::encryptable::Encryptable,
};

use super::user_dto::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub account_id: Uuid,
    pub id: Uuid, // Owner of the account
    pub balance: EncryptedField<f64>,
    pub created_at: NaiveDateTime,
}

impl Account {
    pub fn new(user: &User, balance: f64) -> Result<Self, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(Self {
            account_id: Uuid::now_v7(),
            id: user.id,
            balance: balance.encrypt(&key)?,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn get_balance(&self, user: &User) -> Result<f64, anyhow::Error> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(&user.encryption_key, &master_key)?;
        Ok(f64::decrypt(&self.balance, &key)?)
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
