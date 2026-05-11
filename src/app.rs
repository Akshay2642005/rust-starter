use std::{net::SocketAddr, sync::Arc};

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
}

impl Server {
    pub async fn run(self) -> Result<()> {
        let addr = self
            .listener
            .local_addr()
            .context("could not read local address")?;
        info!(%addr, "http server started");

        axum::serve(
            self.listener,
            self.app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server error")?;

        info!("server shut down cleanly");
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
        let store = SeaOrmStore::connect_and_migrate(&self.cfg)
            .await
            .context("failed to connect to database")?;

        store.ping().await.context("database ping failed")?;

        let state = AppState::new(Arc::clone(&self.cfg), store);
        let app = build_router(state, &self.cfg);

        let addr: SocketAddr =
            format!("{}:{}", self.cfg.server.host, self.cfg.server.port).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        Ok(Server { listener, app })
    }
}

fn build_router(state: AppState, cfg: &Config) -> Router {
    let path_prefix = cfg.server.path_prefix.clone();
    registry::set_route_prefix(&path_prefix);

    let router = registry::install_routes(Router::<AppState>::new()).with_state(state);

    middleware::apply(router, cfg)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl-C"),
        _ = terminate => info!("received SIGTERM"),
    }
}
