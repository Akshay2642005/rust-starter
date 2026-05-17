use crate::{
    auth::BetterAuthService,
    middleware,
    openapi::{build_docs_html, serve_merged_auth_spec},
    registry,
    state::AppState,
};
use anyhow::{Context, Result};
use axum::{Router, routing::get};
use axum_prometheus::PrometheusMetricLayer;
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
        let Server { listener, app, db } = self;
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
    let (prometheus_layer, metrics_handle) = PrometheusMetricLayer::pair();

    let auth_service = state.auth.router().into_service();
    let auth_path = registry::join_paths(&cfg.server.path_prefix, &cfg.auth.path_prefix);
    let auth_rate_limit_cfg = middleware::auth_rate_limit_layer();
    let global_rate_limit_cfg = middleware::global_rate_limit_layer();

    let auth_router =
        Router::new()
            .fallback_service(auth_service)
            .layer(middleware::GovernorLayer {
                config: auth_rate_limit_cfg,
            });

    let mut api_router = registry::install_routes(
        Router::<AppState>::new(),
        &cfg.server.path_prefix,
        state.clone(),
    )
    .nest(&auth_path, auth_router)
    .with_state(state);

    // In non-development, block the better-auth OpenAPI reference endpoint.
    if cfg.primary.env != "development" {
        let reference_path = format!("{}/reference", auth_path.trim_end_matches('/'));
        api_router = api_router.route(
            &format!("{reference_path}/{{*path}}"),
            axum::routing::any(|| async { axum::http::StatusCode::NOT_FOUND }),
        );
    }

    let router = if cfg.primary.env == "development" {
        let docs_html = build_docs_html(cfg);
        let auth_openapi_url = format!(
            "http://localhost:{}{}{}/reference/openapi.json",
            cfg.server.port,
            cfg.server.path_prefix.trim_end_matches('/'),
            cfg.auth.path_prefix.trim_end_matches('/'),
        );
        let docs_router = Router::new()
            .route(
                "/docs",
                get(move || async move { axum::response::Html(docs_html) }),
            )
            .route(
                "/docs/auth-openapi.json",
                get(move || async move { serve_merged_auth_spec(auth_openapi_url).await }),
            );
        api_router.merge(docs_router)
    } else {
        api_router
    };

    let metrics_router = Router::new().route(
        "/metrics",
        get(move || async move { metrics_handle.render() }),
    );

    middleware::apply(router.merge(metrics_router), cfg)
        .layer(middleware::GovernorLayer {
            config: global_rate_limit_cfg,
        })
        .layer(prometheus_layer)
}
