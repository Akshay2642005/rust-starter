use std::{net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use axum::{Router, http::HeaderName, routing::get};
use configuration::Config;
use seaorm::SeaOrmStore;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;

use crate::state::AppState;

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

        info!("database connection verified");

        let state = AppState::new(Arc::clone(&self.cfg), store);
        let app = build(state);

        let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("failed to bind TCP listener to {addr}"))?;
        info!(%addr, "tcp listener bound — ready to serve");

        Ok(Server { listener, app })
    }
}

pub fn build(state: AppState) -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");

    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/healthz", get(|| async { "ok" }))
        // .nest("/api/v1", api_router(state.clone()))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    request_id_header.clone(),
                    MakeRequestUuid,
                ))
                .layer(PropagateRequestIdLayer::new(request_id_header))
                .layer(RequestBodyLimitLayer::new(5 * 1024 * 1024))
                .layer(TraceLayer::new_for_http()),
        )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl-C"),
        _ = terminate => info!("received SIGTERM"),
    }
}
