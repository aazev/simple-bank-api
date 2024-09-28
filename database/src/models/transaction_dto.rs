use crate::{
    decrypt_user_key, load_master_key, structs::encrypted_field::EncryptedField,
    traits::encryptable::Encryptable,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
#[sqlx(type_name = "transaction_operation", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TransactionOperation {
    Deposit,
    Fee,
    Interest,
    Payment,
    Transfer,
    Withdrawal,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub operation: TransactionOperation,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Uuid,
    pub amount: EncryptedField<f64>,
    pub created_at: NaiveDateTime,
}

impl Transaction {
    pub fn new(
        from_account_id: Option<Uuid>,
        to_account_id: Uuid,
        operation: TransactionOperation,
        amount: f64,
        user_key: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(user_key, &master_key)?;
        Ok(Transaction {
            id: Uuid::new_v4(),
            operation,
            from_account_id,
            to_account_id,
            amount: amount.encrypt(&key)?,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }

    pub fn get_amount(&self, user_key: &[u8]) -> Result<f64, Box<dyn std::error::Error>> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(user_key, &master_key)?;
        Ok(f64::decrypt(&self.amount, &key)?)
    }
}

#[derive(Debug, Deserialize)]
pub struct TransactionCreate {
    pub operation: TransactionOperation,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Uuid,
    pub amount: f64,
}

impl TransactionCreate {
    pub fn to_transaction(&self, user_key: &[u8]) -> anyhow::Result<Transaction> {
        let master_key = load_master_key()?;
        let key = decrypt_user_key(user_key, &master_key)?;
        Ok(Transaction {
            id: Uuid::new_v4(),
            operation: self.operation.clone(),
            from_account_id: self.from_account_id,
            to_account_id: self.to_account_id,
            amount: self.amount.encrypt(&key)?,
            created_at: chrono::Utc::now().naive_utc(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct TransactionModel {
    pub id: Uuid,
    pub operation: TransactionOperation,
    pub from_account_id: Option<Uuid>,
    pub to_account_id: Uuid,
    pub amount: f64,
    pub created_at: NaiveDateTime,
}

impl TransactionModel {
    pub fn from_dto(
        transaction: &Transaction,
        user_key: &[u8],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let master = load_master_key()?;
        let key = decrypt_user_key(user_key, &master)?;
        Ok(TransactionModel {
            id: transaction.id,
            operation: transaction.operation.clone(),
            from_account_id: transaction.from_account_id,
            to_account_id: transaction.to_account_id,
            amount: f64::decrypt(&transaction.amount, &key)?,
            created_at: transaction.created_at,
        })
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
        let user = User::new(
            "test".to_string(),
            "test".to_string(),
            Some(true),
            Some("test".to_string()),
        )
        .expect("User creation failed");
        let amount = 250.0;

        let transaction = Transaction::new(
            Some(from_account_id),
            to_account_id,
            TransactionOperation::Deposit,
            amount,
            &user.encryption_key,
        )
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
        let user = User::new(
            "test".to_string(),
            "test".to_string(),
            Some(true),
            Some("test".to_string()),
        )
        .expect("User creation failed");
        let wrong_user = User::new(
            "test".to_string(),
            "test".to_string(),
            Some(true),
            Some("test".to_string()),
        )
        .expect("User creation failed");
        let amount = 250.0;

        let transaction = Transaction::new(
            Some(from_account_id),
            to_account_id,
            TransactionOperation::Deposit,
            amount,
            &user.encryption_key,
        )
        .expect("Transaction creation failed");

        let result = transaction.get_amount(&wrong_user.encryption_key);

        assert!(
            result.is_err(),
            "Decrypting amount with wrong key should fail"
        );
    }
}
