use std::sync::Arc;

use configuration::Config;
use seaorm::SeaOrmStore;

#[derive(Clone)]
pub struct AppState {
    pub _config: Arc<Config>,
    pub _db: SeaOrmStore,
}

impl AppState {
    pub fn new(config: Arc<Config>, db: SeaOrmStore) -> Self {
        Self {
            _config: config,
            _db: db,
        }
    }
}
