use std::sync::Arc;

use crate::auth::BetterAuthService;
use axum::extract::FromRef;
use better_auth::{BetterAuth, adapters::SqlxAdapter};
use configuration::Config;
use seaorm::SeaOrmStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SeaOrmStore,
    pub auth: BetterAuthService,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: SeaOrmStore, auth: BetterAuthService) -> Self {
        Self { config, db, auth }
    }
}

impl FromRef<AppState> for Arc<BetterAuth<SqlxAdapter>> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.inner()
    }
}
