use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{de::DeserializeOwned, Serialize};

use crate::structs::encrypted_field::EncryptedField;

pub trait Encryptable {
    fn encrypt(&self, key: &[u8]) -> EncryptedField<Self>
    where
        Self: Sized + Serialize,
    {
        // Serialize the data
        let serialized_data = bincode::serialize(&self).unwrap();

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();

        // Generate a unique nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, serialized_data.as_ref()).unwrap();

        EncryptedField::new(nonce_bytes.to_vec(), ciphertext)
    }

    fn decrypt(encrypted_field: &EncryptedField<Self>, key: &[u8]) -> Self
    where
        Self: Sized + DeserializeOwned,
    {
        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key).unwrap();

        // Use the nonce from the encrypted field
        let nonce = Nonce::from_slice(&encrypted_field.nonce);

        // Decrypt the data
        let decrypted_data = cipher
            .decrypt(nonce, encrypted_field.ciphertext.as_ref())
            .unwrap();

        // Deserialize the data
        bincode::deserialize(&decrypted_data).unwrap()
    }
}

// Implement Encryptable for all types that satisfy the trait bounds
impl<T> Encryptable for T where T: Serialize + DeserializeOwned {}
