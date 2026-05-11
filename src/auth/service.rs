use crate::auth::hasher::RuntimeArgon2Hasher;
use anyhow::Result;
use axum::Router;

use better_auth::adapters::{PoolConfig, SqlxAdapter};
use better_auth::handlers::AxumIntegration;
use better_auth::plugins::{
    AccountManagementPlugin, EmailPasswordPlugin, PasswordManagementPlugin, SessionManagementPlugin,
};
use better_auth::{Argon2Config, AuthBuilder, AuthConfig as BetterAuthConfig, BetterAuth};
use std::sync::Arc;
use std::time::Duration;

pub type BetterAuthInstance = BetterAuth<SqlxAdapter>;

#[derive(Clone)]
pub struct BetterAuthService {
    inner: Arc<BetterAuthInstance>,
}

impl BetterAuthService {
    pub async fn build(config: &configuration::Config) -> Result<Self> {
        let password_hasher = Self::build_password_hasher(config);
        let adapter =
            SqlxAdapter::with_config(&config.store.url, Self::build_pool_config(config)).await?;

        let auth = Arc::new(
            AuthBuilder::new(Self::build_auth_config(config))
                .database(adapter)
                .plugin(
                    EmailPasswordPlugin::new()
                        .enable_signup(config.auth.enable_signup)
                        .password_min_length(usize::from(config.auth.password_min_length))
                        .password_hasher(password_hasher),
                )
                .plugin(SessionManagementPlugin::new())
                .plugin(PasswordManagementPlugin::new())
                .plugin(AccountManagementPlugin::new())
                .build()
                .await?,
        );

        tracing::info!("better-auth initialized");

        Ok(Self { inner: auth })
    }

    pub(crate) fn router(&self) -> Router {
        let auth = Arc::clone(&self.inner);
        auth.clone().axum_router().with_state(auth)
    }

    fn build_auth_config(config: &configuration::Config) -> BetterAuthConfig {
        BetterAuthConfig::new(config.auth.secret.clone())
            .base_url(config.auth.base_url.clone())
            .base_path(config.auth.path_prefix.clone())
            .password_min_length(usize::from(config.auth.password_min_length))
    }

    fn build_password_hasher(config: &configuration::Config) -> Arc<RuntimeArgon2Hasher> {
        Arc::new(RuntimeArgon2Hasher::new(Argon2Config {
            memory_cost: config.auth.argon2.memory_cost,
            time_cost: config.auth.argon2.time_cost,
            parallelism: config.auth.argon2.parallelism,
        }))
    }

    fn build_pool_config(config: &configuration::Config) -> PoolConfig {
        PoolConfig {
            max_connections: config.store.max_connections,
            min_connections: config.store.min_connections,
            acquire_timeout: Duration::from_secs(config.store.acquire_timeout_secs),
            idle_timeout: Some(Duration::from_secs(config.store.idle_timeout_secs)),
            max_lifetime: Some(Duration::from_secs(config.store.max_lifetime_secs)),
        }
    }
}
