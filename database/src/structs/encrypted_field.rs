use serde::{Deserialize, Serialize};

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
