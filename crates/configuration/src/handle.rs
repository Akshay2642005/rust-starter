use crate::Config;
use arc_swap::ArcSwap;
use std::sync::Arc;

#[derive(Clone)]
pub struct ConfigHandle {
    inner: Arc<ArcSwap<Config>>,
}

impl ConfigHandle {
    pub fn new(initial: Config) -> Self {
        Self {
            inner: Arc::new(ArcSwap::from_pointee(initial)),
        }
    }

    pub fn load(&self) -> Arc<Config> {
        self.inner.load_full()
    }

    pub fn swap(&self, new_config: Config) {
        self.inner.store(Arc::new(new_config));
    }
}
