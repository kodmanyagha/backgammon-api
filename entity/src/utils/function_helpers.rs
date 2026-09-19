use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::{distr::Alphanumeric, seq::SliceRandom, RngExt};
use sha2::{Digest, Sha256};

// Best value for encryption and unique file names.
pub const UNIQUE_KEY_LEN: usize = 32;

// Best value for redis and mysql operations.
pub const OPTIMAL_RPP: u64 = 1_000;

pub fn crypto_random_str(len: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .map(|item| item.to_ascii_lowercase())
        .collect()
}

/// Returns the lowercase hex encoded SHA-256 digest of `input`.
pub fn sha256_hex(input: &str) -> String {
    Sha256::digest(input.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn random_between(min: u64, max: u64) -> u64 {
    rand::rng().random_range(min..max)
}

pub fn is_alphanumeric(s: &str) -> bool {
    s.chars().all(|c| c.is_alphanumeric())
}

pub fn is_valid_len(s: &str, len: usize) -> bool {
    s.len() == len
}

pub fn get_only_numbers(input_str: &str) -> String {
    input_str.chars().filter(|c| c.is_ascii_digit()).collect()
}

fn derive_encryption_key(secret: &str) -> Key<Aes256Gcm> {
    let digest = Sha256::digest(secret.as_bytes());
    Key::<Aes256Gcm>::clone_from_slice(&digest)
}

/// Encrypts `plain_text` with AES-256-GCM using `secret` as the key material and returns
/// the base64-encoded `nonce || ciphertext`.
pub fn encrypt_base64(plain_text: &str, secret: &str) -> anyhow::Result<String> {
    let cipher = Aes256Gcm::new(&derive_encryption_key(secret));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plain_text.as_bytes())
        .map_err(|err| anyhow::anyhow!("Encryption failed: {err}"))?;

    let mut payload = nonce.to_vec();
    payload.extend_from_slice(&ciphertext);

    Ok(STANDARD.encode(payload))
}

/// Decrypts a base64 `nonce || ciphertext` payload produced by [`encrypt_base64`] using
/// `secret` as the key material.
pub fn decrypt_base64(cipher_text_b64: &str, secret: &str) -> anyhow::Result<String> {
    let payload = STANDARD.decode(cipher_text_b64)?;

    if payload.len() < 12 {
        return Err(anyhow::anyhow!("Invalid encrypted payload"));
    }

    let (nonce_bytes, ciphertext) = payload.split_at(12);
    let nonce = Nonce::clone_from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new(&derive_encryption_key(secret));
    let plain_text = cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|err| anyhow::anyhow!("Decryption failed: {err}"))?;

    Ok(String::from_utf8(plain_text)?)
}

pub fn random_cellphone() -> u64 {
    let mut prefixes: Vec<u64> = vec![5370000000, 5320000000, 5420000000, 5520000000, 5550000000];
    prefixes.shuffle(&mut rand::rng());

    let random_part = random_between(1000000, 9999999);
    let phone_number = prefixes.first().cloned().unwrap_or_default() + random_part;

    phone_number
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_matches_the_known_test_vector() {
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn sha256_hex_is_always_64_lowercase_hex_chars() {
        let digest = sha256_hex("any device secret");

        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)));
    }
}
