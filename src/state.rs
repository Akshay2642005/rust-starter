use std::sync::Arc;

use crate::auth::BetterAuthService;
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
