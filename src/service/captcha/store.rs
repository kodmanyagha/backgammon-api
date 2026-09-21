use crate::service::redis_service::redis_service::RedisService;

const CHALLENGE_KEY_PREFIX: &str = "captcha:challenge:";
const VERIFICATION_KEY_PREFIX: &str = "captcha:verification:";

fn challenge_key(hash: &str) -> String {
    format!("{CHALLENGE_KEY_PREFIX}{hash}")
}

fn verification_key(hash: &str) -> String {
    format!("{VERIFICATION_KEY_PREFIX}{hash}")
}

pub async fn save_challenge(
    redis: &RedisService,
    hash: &str,
    correct_answer: u8,
    ttl_seconds: u64,
) -> anyhow::Result<()> {
    redis
        .set_with_ttl(&challenge_key(hash), &correct_answer.to_string(), ttl_seconds)
        .await
}

pub async fn take_challenge_answer(redis: &RedisService, hash: &str) -> anyhow::Result<Option<u8>> {
    let stored = redis.get_del(&challenge_key(hash)).await?;

    Ok(stored.and_then(|answer| answer.parse::<u8>().ok()))
}

pub async fn save_verification(
    redis: &RedisService,
    hash: &str,
    expires_at: i64,
    ttl_seconds: u64,
) -> anyhow::Result<()> {
    redis
        .set_with_ttl(&verification_key(hash), &expires_at.to_string(), ttl_seconds)
        .await
}

pub async fn is_verified(redis: &RedisService, hash: &str) -> anyhow::Result<bool> {
    redis.exists(&verification_key(hash)).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenges_and_verifications_live_under_separate_keys() {
        let hash = "a".repeat(40);

        assert_eq!(challenge_key(&hash), format!("captcha:challenge:{hash}"));
        assert_eq!(verification_key(&hash), format!("captcha:verification:{hash}"));
        assert_ne!(challenge_key(&hash), verification_key(&hash));
    }
}
