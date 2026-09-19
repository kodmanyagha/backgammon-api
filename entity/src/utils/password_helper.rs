use anyhow::anyhow;
use bcrypt::{hash, verify, DEFAULT_COST};

pub fn verify_password(raw_password: &str, hash: &str) -> bool {
    verify(raw_password, hash)
        .map_err(|e| anyhow!("{e}"))
        .unwrap_or(false)
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    hash(password, DEFAULT_COST).map_err(|e| anyhow!("{e}"))
}
