use crate::{auth::BetterAuthService, middleware, openapi::{AppApiDoc, SystemApiDoc}, registry, state::AppState};
use anyhow::{Context, Result};
use axum::{Router, routing::get};
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
    let auth_service = state.auth.router().into_service();
    let auth_path = registry::join_paths(&cfg.server.path_prefix, &cfg.auth.path_prefix);

    let mut api_router = registry::install_routes(
        Router::<AppState>::new(),
        &cfg.server.path_prefix,
        state.clone(),
    )
    .nest_service(&auth_path, auth_service)
    .with_state(state);

    // In non-development, block the better-auth OpenAPI reference endpoint.
    if cfg.primary.env != "development" {
        let reference_path = format!("{}/reference", auth_path.trim_end_matches('/'));
        api_router = api_router.route(
            &format!("{reference_path}/{{*path}}"),
            axum::routing::any(|| async { axum::http::StatusCode::NOT_FOUND }),
        );
    }

    // Docs routes are stateless — build separately and merge after state is applied.
    // Only mounted in development; not available in production.
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
    middleware::apply(router, cfg)
}

async fn serve_merged_auth_spec(auth_openapi_url: String) -> axum::response::Response {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    let app_schemas = extract_app_schemas();

    let spec_json = match reqwest::get(auth_openapi_url).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
            }
        },
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
        }
    };

    let mut spec: serde_json::Value = match serde_json::from_str(&spec_json) {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
        }
    };

    if let (Some(spec_obj), Some(app_obj)) = (spec.as_object_mut(), app_schemas.as_object()) {
        // Ensure components.schemas exists
        if !spec_obj.contains_key("components") {
            spec_obj.insert("components".into(), serde_json::json!({"schemas": {}}));
        }
        let components = spec_obj["components"].as_object_mut().unwrap();
        if !components.contains_key("schemas") {
            components.insert("schemas".into(), serde_json::json!({}));
        }
        let schemas = components["schemas"].as_object_mut().unwrap();
        for (k, v) in app_obj {
            schemas.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }

    axum::Json(spec).into_response()
}

fn extract_app_schemas() -> serde_json::Value {
    use utoipa::OpenApi;
    let mut schemas = serde_json::Map::new();
    for spec in [AppApiDoc::openapi(), SystemApiDoc::openapi()] {
        if let Ok(json) = serde_json::to_value(&spec) {
            if let Some(obj) = json
                .get("components")
                .and_then(|c| c.get("schemas"))
                .and_then(|s| s.as_object())
            {
                for (k, v) in obj {
                    schemas.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
        }
    }
    serde_json::Value::Object(schemas)
}

fn build_docs_html(cfg: &Config) -> String {
    use utoipa::OpenApi;
    let app_spec =
        serde_json::to_string(&AppApiDoc::openapi()).expect("failed to serialize App API spec");
    let system_spec =
        serde_json::to_string(&SystemApiDoc::openapi()).expect("failed to serialize System API spec");

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
          {{ title: 'App API', content: {app_spec} }},
          {{ title: 'System API', content: {system_spec} }},
          {{
            title: 'Auth API',
            url: '/docs/auth-openapi.json',
            servers: [{{ url: '{auth_base_url}' }}]
          }}
        ]
      }});
    </script>
  </body>
</html>"#
    )
}
