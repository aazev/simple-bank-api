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
            amount: amount.encrypt(user_key),
            timestamp: chrono::Utc::now().naive_utc(),
        })
    }
}
