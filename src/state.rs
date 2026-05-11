use std::sync::Arc;

use configuration::Config;
use seaorm::SeaOrmStore;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SeaOrmStore,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: SeaOrmStore) -> Self {
        Self { config, db }
    }
}
