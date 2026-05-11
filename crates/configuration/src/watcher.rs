use crate::loader::load_config;
use crate::{Config, ConfigHandle};
use anyhow::{Context, Result};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;

pub struct ConfigManager {
    env_name: String,
    config_dir: PathBuf,
    handle: ConfigHandle,
}

impl ConfigManager {
    pub fn load_initial(
        env_name: &str,
        config_dir: impl AsRef<Path>,
    ) -> Result<(Self, ConfigHandle)> {
        let config_dir = config_dir.as_ref().to_path_buf();
        let initial = load_config(env_name, &config_dir)
            .with_context(|| format!("failed to load config from {}", config_dir.display()))?;
        let handle = ConfigHandle::new(initial);

        Ok((
            Self {
                env_name: env_name.to_string(),
                config_dir,
                handle: handle.clone(),
            },
            handle,
        ))
    }

    pub fn reload_once(&self) -> Result<Config> {
        load_config(&self.env_name, &self.config_dir)
    }

    pub fn spawn_watcher(&self, mut shutdown: broadcast::Receiver<()>) -> JoinHandle<()> {
        let env_name = self.env_name.clone();
        let config_dir = self.config_dir.clone();
        let handle = self.handle.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel();

            let mut watcher = match new_watcher(tx) {
                Ok(watcher) => watcher,
                Err(error) => {
                    tracing::error!(%error, "failed to create config watcher");
                    return;
                }
            };

            if let Err(error) = watcher.watch(&config_dir, RecursiveMode::NonRecursive) {
                tracing::error!(
                    %error,
                    path = %config_dir.display(),
                    "failed to watch config directory"
                );
                return;
            }

            tracing::info!(path = %config_dir.display(), "config watcher started");

            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        break;
                    }
                    event = rx.recv() => {
                        if event.is_none() {
                            break;
                        }

                        tokio::time::sleep(Duration::from_millis(250)).await;

                        while rx.try_recv().is_ok() {}

                        match load_config(&env_name, &config_dir) {
                            Ok(next) => {
                                handle.swap(next);
                                tracing::info!("configuration reloaded");
                            }
                            Err(error) => {
                                tracing::error!(
                                    %error,
                                    "configuration reload failed; keeping last known good config"
                                );
                            }
                        }
                    }
                }
            }
        })
    }
}

fn new_watcher(tx: mpsc::UnboundedSender<()>) -> notify::Result<RecommendedWatcher> {
    notify::recommended_watcher(move |result: notify::Result<notify::Event>| match result {
        Ok(event) if is_relevant_event(&event.kind) && touches_config_file(&event.paths) => {
            let _ = tx.send(());
        }
        Ok(_) => {}
        Err(error) => {
            tracing::error!(%error, "config watcher emitted an error");
        }
    })
}

fn is_relevant_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn touches_config_file(paths: &[PathBuf]) -> bool {
    paths.iter().any(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext, "toml" | "yml" | "yaml"))
    })
}
