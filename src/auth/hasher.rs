use anyhow::{Result, anyhow};
use argon2::password_hash::{PasswordHash, SaltString, rand_core::OsRng};
use argon2::{Algorithm, Argon2, Params, PasswordHasher as _, PasswordVerifier, Version};
use async_trait::async_trait;
use better_auth::Argon2Config;
use better_auth::plugins::PasswordHasher;

#[derive(Clone, Debug)]
pub struct RuntimeArgon2Hasher {
    config: Argon2Config,
}

impl RuntimeArgon2Hasher {
    #[must_use]
    pub fn new(config: Argon2Config) -> Self {
        Self { config }
    }

    fn build_argon2(&self) -> Result<Argon2<'static>> {
        let params = Params::new(
            self.config.memory_cost,
            self.config.time_cost,
            self.config.parallelism,
            None,
        )
        .map_err(|error| anyhow!("invalid argon2 config: {error}"))?;

        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }
}

#[async_trait]
impl PasswordHasher for RuntimeArgon2Hasher {
    async fn hash(&self, password: &str) -> better_auth::AuthResult<String> {
        let password = password.to_owned();
        let argon2 = self
            .build_argon2()
            .map_err(|error| better_auth::AuthError::PasswordHash(error.to_string()))?;

        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);

            let hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|error| {
                    better_auth::AuthError::PasswordHash(format!(
                        "failed to hash password: {error}"
                    ))
                })?;

            Ok(hash.to_string())
        })
        .await
        .map_err(|error| {
            better_auth::AuthError::PasswordHash(format!("password task failed: {error}"))
        })?
    }

    async fn verify(&self, hash: &str, password: &str) -> better_auth::AuthResult<bool> {
        let password = password.to_owned();
        let hash = hash.to_owned();
        let argon2 = self
            .build_argon2()
            .map_err(|error| better_auth::AuthError::PasswordHash(error.to_string()))?;

        tokio::task::spawn_blocking(move || {
            let parsed_hash = PasswordHash::new(&hash).map_err(|error| {
                better_auth::AuthError::PasswordHash(format!("invalid password hash: {error}"))
            })?;

            Ok(argon2
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok())
        })
        .await
        .map_err(|error| {
            better_auth::AuthError::PasswordHash(format!("password task failed: {error}"))
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Argon2Config {
        Argon2Config {
            memory_cost: 4096,
            time_cost: 2,
            parallelism: 1,
        }
    }

    #[tokio::test]
    async fn hash_and_verify_roundtrip_succeeds() {
        let hasher = RuntimeArgon2Hasher::new(test_config());
        let password = "correct-horse-battery-staple";

        let hash = hasher.hash(password).await.unwrap();

        assert!(hasher.verify(&hash, password).await.unwrap());
        assert!(!hasher.verify(&hash, "wrong-password").await.unwrap());
    }

    #[tokio::test]
    async fn invalid_hash_fails_cleanly() {
        let hasher = RuntimeArgon2Hasher::new(test_config());
        let error = hasher
            .verify("not-a-real-hash", "secret")
            .await
            .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("invalid password hash"));
    }
}
