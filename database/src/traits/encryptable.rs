use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{de::DeserializeOwned, Serialize};

use crate::structs::encrypted_field::{EncryptedField, EncryptionError};

pub trait Encryptable {
    fn encrypt(&self, key: &[u8]) -> Result<EncryptedField<Self>, EncryptionError>
    where
        Self: Sized + Serialize,
    {
        // Serialize the data
        let serialized_data = bincode::serialize(&self)?;

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key)?;

        // Generate a unique nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, serialized_data.as_ref())?;

        Ok(EncryptedField::new(nonce_bytes.to_vec(), ciphertext))
    }

    fn decrypt(encrypted_field: &EncryptedField<Self>, key: &[u8]) -> Result<Self, EncryptionError>
    where
        Self: Sized + DeserializeOwned,
    {
        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(key)?;

        // Use the nonce from the encrypted field
        let nonce = Nonce::from_slice(&encrypted_field.nonce);

        // Decrypt the data
        let decrypted_data = cipher.decrypt(nonce, encrypted_field.ciphertext.as_ref())?;

        // Deserialize the data
        Ok(bincode::deserialize(&decrypted_data).unwrap())
    }
}

// Implement Encryptable for all types that satisfy the trait bounds
impl<T> Encryptable for T where T: Serialize + DeserializeOwned {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_f64() {
        let key = [0u8; 32]; // Use a fixed key for testing
        let data: f64 = 42.0;

        // Encrypt the data
        let encrypted_field = data.encrypt(&key).expect("Encryption failed");

        // Decrypt the data
        let decrypted_data = f64::decrypt(&encrypted_field, &key).expect("Decryption failed");

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = [0u8; 32]; // Use a fixed key for testing
        let data = String::from("Hello, world!");

        // Encrypt the data
        let encrypted_field = data.encrypt(&key).expect("Encryption failed");

        // Decrypt the data
        let decrypted_data = String::decrypt(&encrypted_field, &key).expect("Decryption failed");

        assert_eq!(data, decrypted_data);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key = [0u8; 32]; // Original key
        let wrong_key = [1u8; 32]; // Different key
        let data: f64 = 42.0;

        // Encrypt the data
        let encrypted_field = data.encrypt(&key).expect("Encryption failed");

        // Attempt to decrypt with the wrong key
        let result = f64::decrypt(&encrypted_field, &wrong_key);

        assert!(result.is_err(), "Decryption should fail with wrong key");
    }

    #[test]
    fn test_encrypt_with_invalid_key_length() {
        let key = [0u8; 16]; // Incorrect key length (should be 32 bytes)
        let data: f64 = 42.0;

        let result = data.encrypt(&key);

        assert!(
            matches!(result, Err(EncryptionError::InvalidKeyLength(_))),
            "Encryption should fail with InvalidKeyLength error"
        );
    }
}
