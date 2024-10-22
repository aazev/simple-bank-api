use cipher::InvalidLength;
use serde::{ser::StdError, Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedField<T> {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
    #[serde(skip)]
    _marker: std::marker::PhantomData<T>,
}

impl<T> EncryptedField<T> {
    pub fn new(nonce: Vec<u8>, ciphertext: Vec<u8>) -> Self {
        EncryptedField {
            nonce,
            ciphertext,
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] aes_gcm::Error),
    #[error("Invalid key length: {0}")]
    InvalidKeyLength(#[from] InvalidLength),
}

macro_rules! impl_sqlx_for_encrypted_field {
    ($t:ty) => {
        use sqlx::encode::IsNull;
        use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
        use sqlx::{Decode, Encode, Postgres, Type};
        use std::error::Error;

        impl Type<Postgres> for EncryptedField<$t> {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("BYTEA")
            }
        }

        impl<'q> Encode<'q, Postgres> for EncryptedField<$t> {
            fn encode_by_ref(
                &self,
                buf: &mut PgArgumentBuffer,
            ) -> Result<IsNull, Box<(dyn StdError + Send + Sync + 'static)>> {
                match bincode::serialize(self) {
                    Ok(serialized) => {
                        <Vec<u8> as Encode<Postgres>>::encode_by_ref(&serialized, buf)
                    }
                    Err(_) => Ok(IsNull::Yes),
                }
            }

            fn size_hint(&self) -> usize {
                match bincode::serialized_size(self) {
                    Ok(size) => size as usize,
                    Err(_) => 0,
                }
            }
        }

        impl<'r> Decode<'r, Postgres> for EncryptedField<$t> {
            fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
                let bytes = <Vec<u8> as Decode<Postgres>>::decode(value)?;
                let encrypted_field: EncryptedField<$t> = bincode::deserialize(&bytes)?;
                Ok(encrypted_field)
            }
        }
    };
}

impl_sqlx_for_encrypted_field!(f64);
