use crate::{auth::BetterAuthService, middleware, registry, state::AppState};
use anyhow::{Context, Result};
use axum::Router;
use configuration::Config;
use seaorm::SeaOrmStore;
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;

pub struct Server {
    listener: TcpListener,
    app: Router,
    db: SeaOrmStore,
}

impl Server {
    pub async fn run<S>(self, shutdown: S) -> Result<()>
    where
        S: Future<Output = ()> + Send + 'static,
    {
        let Server {
            listener, app, db, ..
        } = self;
        let addr = listener
            .local_addr()
            .context("could not read local address")?;
        info!(%addr, "http server started");

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown)
        .await
        .context("server error")?;

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

        let auth = BetterAuthService::build(&self.cfg)
            .await
            .context("failed to initialize auth")?;

        let state = AppState::new(Arc::clone(&self.cfg), store.clone(), auth);
        let app = build_router(state, &self.cfg);

        let addr: SocketAddr =
            format!("{}:{}", self.cfg.server.host, self.cfg.server.port).parse()?;
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        Ok(Server {
            listener,
            app,
            db: store,
        })
    }
}

fn build_router(state: AppState, cfg: &Config) -> Router {
    let auth_service = state.auth.router().into_service();
    let auth_path = registry::join_paths(&cfg.server.path_prefix, &cfg.auth.path_prefix);
    let router = registry::install_routes(Router::<AppState>::new(), &cfg.server.path_prefix)
        .nest_service(&auth_path, auth_service)
        .with_state(state);
    middleware::apply(router, cfg)
}
