use crate::{auth::BetterAuthService, middleware, openapi::ApiDoc, registry, state::AppState};
use anyhow::{Context, Result};
use axum::{Router, routing::get};
use configuration::Config;
use seaorm::SeaOrmStore;
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tracing::info;
use utoipa::OpenApi;

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
    let auth_service = state.auth.router().into_service();
    let auth_path = registry::join_paths(&cfg.server.path_prefix, &cfg.auth.path_prefix);

    let api_router = registry::install_routes(
        Router::<AppState>::new(),
        &cfg.server.path_prefix,
        state.clone(),
    )
    .nest_service(&auth_path, auth_service)
    .with_state(state);

    // Docs routes are stateless — build separately and merge after state is applied.
    // The app spec is inlined at startup so Scalar doesn't need to fetch /openapi.json.
    let docs_html = build_docs_html(cfg);
    let docs_router = Router::new().route(
        "/docs",
        get(move || async move { axum::response::Html(docs_html) }),
    );

    let router = api_router.merge(docs_router);
    middleware::apply(router, cfg)
}

fn build_docs_html(cfg: &Config) -> String {
    let spec = serde_json::to_string(&ApiDoc::openapi()).expect("failed to serialize OpenAPI spec");

    // BetterAuth's OpenAPI spec may have an incorrect servers entry.
    // Override with the actual URL: server base_url + path_prefix + auth path_prefix.
    let auth_base_url = format!(
        "http://localhost:{}{}{}",
        cfg.server.port,
        cfg.server.path_prefix.trim_end_matches('/'),
        cfg.auth.path_prefix,
    );

    format!(
        r#"<!doctype html>
<html>
  <head>
    <title>API Reference</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <div id="app"></div>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
    <script>
      Scalar.createApiReference('#app', {{
        theme: 'default',
        sources: [
          {{ title: 'App API', content: {spec} }},
          {{
            title: 'Auth API',
            url: '/api/v1/auth/reference/openapi.json',
            servers: [{{ url: '{auth_base_url}' }}]
          }}
        ]
      }});
    </script>
  </body>
</html>"#
    )
}
