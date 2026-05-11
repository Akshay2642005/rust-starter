use std::{future::IntoFuture, net::SocketAddr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use axum::Router;
use configuration::Config;

use seaorm::SeaOrmStore;
use tokio::net::TcpListener;
use tracing::info;

use crate::{middleware, registry, state::AppState};

pub struct Server {
    listener: TcpListener,
    app: Router,
    shutdown_timeout: Duration,
    min_graceful_shutdown: Duration,
    db: SeaOrmStore,
}

impl Server {
    pub async fn run(self, mut shutdown: tokio::sync::broadcast::Receiver<()>) -> Result<()> {
        let addr = self
            .listener
            .local_addr()
            .context("could not read local address")?;
        info!(%addr, "http server started");

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let server = axum::serve(
            self.listener,
            self.app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        let mut server_handle = tokio::spawn(server.into_future());

        tokio::select! {
            _ = shutdown.recv() => {
                info!("shutdown signal received");
                info!("initializing graceful shutdown");
            }
            result = &mut server_handle => {
                return result
                    .context("server task failed")?
                    .context("server error");
            }
        }

        let _ = shutdown_tx.send(());

        if !self.min_graceful_shutdown.is_zero() {
            tokio::time::sleep(self.min_graceful_shutdown).await;
        }

        match tokio::time::timeout(self.shutdown_timeout, &mut server_handle).await {
            Ok(result) => {
                result
                    .context("server task failed")?
                    .context("server error")?;
            }
            Err(_) => {
                tracing::warn!(
                    timeout_secs = self.shutdown_timeout.as_secs(),
                    "graceful shutdown timed out; forcing exit"
                );
                server_handle.abort();
            }
        }

        let db = self.db;
        if let Err(error) = db.close().await {
            tracing::warn!(%error, "failed to close database connections cleanly");
        }

        Ok(())
    }
}

pub struct ServerBuilder {
    cfg: Arc<Config>,
}

impl ServerBuilder {
    pub fn new(cfg: Arc<Config>) -> Self {
        Self { cfg }
    }

    pub async fn build(self) -> Result<Server> {
        let store = SeaOrmStore::connect(&self.cfg)
            .await
            .context("failed to connect to database")?;

        store.ping().await.context("database ping failed")?;

        let state = AppState::new(Arc::clone(&self.cfg), store.clone());
        let app = build_router(state, &self.cfg);

        let addr: SocketAddr =
            format!("{}:{}", self.cfg.server.host, self.cfg.server.port).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        let shutdown_timeout = Duration::from_secs(self.cfg.server.shutdown_timeout_secs);
        let min_graceful_shutdown = Duration::from_secs(self.cfg.server.min_graceful_shutdown_secs);

        Ok(Server {
            listener,
            app,
            shutdown_timeout,
            min_graceful_shutdown,
            db: store,
        })
    }
}

fn build_router(state: AppState, cfg: &Config) -> Router {
    let path_prefix = cfg.server.path_prefix.clone();
    registry::set_route_prefix(&path_prefix);
    let router = registry::install_routes(Router::<AppState>::new()).with_state(state);
    middleware::apply(router, cfg)
}
