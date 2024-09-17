use crate::{structs::encrypted_field::EncryptedField, traits::encryptable::Encryptable};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: Uuid,
    pub from_account_id: Uuid,
    pub to_account_id: Uuid,
    pub amount: EncryptedField<f64>,
    pub timestamp: NaiveDateTime,
}

impl Transaction {
    pub fn new(
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: f64,
        user_key: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Transaction {
            transaction_id: Uuid::new_v4(),
            from_account_id,
            to_account_id,
            amount: amount.encrypt(user_key)?,
            timestamp: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn get_amount(&self, user_key: &[u8]) -> Result<f64, Box<dyn std::error::Error>> {
        Ok(f64::decrypt(&self.amount, user_key)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::models::user_dto::User;

    use super::*;

    #[test]
    fn test_transaction_creation_and_amount() {
        let from_account_id = Uuid::new_v4();
        let to_account_id = Uuid::new_v4();
        let user = User::new("test".to_string(), "test".to_string(), [0u8; 32].to_vec());
        let amount = 250.0;

        let transaction =
            Transaction::new(from_account_id, to_account_id, amount, &user.encryption_key)
                .expect("Transaction creation failed");

        let decrypted_amount = transaction
            .get_amount(&user.encryption_key)
            .expect("Failed to decrypt amount");

        assert_eq!(amount, decrypted_amount);
    }

    #[test]
    fn test_transaction_amount_with_wrong_key_fails() {
        let from_account_id = Uuid::new_v4();
        let to_account_id = Uuid::new_v4();
        let user = User::new("test".to_string(), "test".to_string(), [0u8; 32].to_vec());
        let wrong_user = User::new("test".to_string(), "test".to_string(), [1u8; 32].to_vec());
        let amount = 250.0;

        let transaction =
            Transaction::new(from_account_id, to_account_id, amount, &user.encryption_key)
                .expect("Transaction creation failed");

        let result = transaction.get_amount(&wrong_user.encryption_key);

        assert!(
            result.is_err(),
            "Decrypting amount with wrong key should fail"
        );
    }
}
