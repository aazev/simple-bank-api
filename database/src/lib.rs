pub mod models;
pub mod structs;
pub mod traits;

use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use cipher::{InvalidLength, KeyInit};
use dotenv::dotenv;
use rand::{rngs::OsRng, RngCore};
use sqlx::{postgres::PgPoolOptions, PgPool};
use thiserror::Error;

use aes_gcm::{aead::Aead, Aes256Gcm, Error as AesError, Nonce};

pub async fn get_database_pool(min: Option<u32>, max: Option<u32>) -> PgPool {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let dedicated: bool = std::env::var("DEDICATED_SERVER")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();
    let cpus = num_cpus::get() as u32;
    // if dedicated then use all cpus, if not, use all but 2 with the minimum being 1
    let min_connections = match dedicated {
        true => cpus,
        // not a dedicated server, so leave 2 cpus for the OS, but keep cpus with a minimum of 1
        false => cpus.saturating_sub(2).max(1),
    };
    let max_connections = cpus * 2;

    PgPoolOptions::new()
        .max_connections(max.unwrap_or(max_connections))
        .min_connections(min.unwrap_or(min_connections))
        .connect(&database_url)
        .await
        .expect("Failed to create pool")
}

#[derive(Debug, Error)]
pub enum KeyManagementError {
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] AesError),
    #[error("Invalid key length: {0}")]
    InvalidKeyLength(#[from] InvalidLength),
}

pub fn encrypt_user_key(user_key: &[u8], master_key: &[u8]) -> Result<Vec<u8>, KeyManagementError> {
    let cipher = Aes256Gcm::new_from_slice(master_key)?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    let ciphertext = cipher.encrypt((&nonce_bytes).into(), user_key)?;

    Ok([nonce_bytes.to_vec(), ciphertext].concat())
}

pub fn decrypt_user_key(
    encrypted_user_key: &[u8],
    master_key: &[u8],
) -> Result<Vec<u8>, KeyManagementError> {
    let cipher = Aes256Gcm::new_from_slice(master_key)?;

    if encrypted_user_key.len() < 12 {
        return Err(KeyManagementError::EncryptionError(aes_gcm::Error.into()));
    }

    let (nonce_bytes, ciphertext) = encrypted_user_key.split_at(12);

    let nonce = Nonce::from_slice(&nonce_bytes);

    let decrypted_user_key = cipher.decrypt(nonce, ciphertext)?;

    Ok(decrypted_user_key)
}

pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    match argon2::PasswordHash::new(hash) {
        Ok(parsed_hash) => Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()),
        Err(_) => Ok(false),
    }
}
