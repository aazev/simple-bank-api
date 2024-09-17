use cipher::InvalidLength;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedField<T> {
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
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
