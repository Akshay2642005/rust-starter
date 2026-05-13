//! OpenAPI 3.1 specification and Scalar UI.
//!
//! Mount with:
//! ```rust,ignore
//! use utoipa_scalar::{Scalar, Servable};
//! let router = router.merge(Scalar::with_url("/docs", ApiDoc::openapi()));
//! ```

use configuration::Config;
use utoipa::OpenApi;

use crate::{
    handlers::{
        system::{__path_health, __path_healthz, __path_livez, __path_readyz, __path_status},
        todos::{
            __path_create_todo, __path_delete_todo, __path_get_todo_by_id, __path_get_todos,
            __path_update_todo,
        },
    },
    response::{
        error::ErrorResponse,
        system::{ComponentStatus, ProbeResponse, StatusChecks, StatusResponse},
        todo::{CreateTodoRequest, TodoResponse, UpdateTodoRequest},
    },
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "App API",
        version = "0.1.0",
        description = "Production-ready Rust backend starter — todo service",
        license(name = "MIT"),
    ),
    paths(create_todo, get_todos, get_todo_by_id, update_todo, delete_todo),
    components(schemas(ErrorResponse, TodoResponse, CreateTodoRequest, UpdateTodoRequest)),
    tags((name = "todos", description = "Todo management endpoints")),
)]
struct AppApiDoc;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "System API",
        version = "0.1.0",
        description = "Health and status endpoints",
        license(name = "MIT"),
    ),
    paths(health, healthz, livez, readyz, status),
    components(schemas(ProbeResponse, ComponentStatus, StatusChecks, StatusResponse)),
    tags((name = "system", description = "Health and status endpoints")),
)]
struct SystemApiDoc;

// =======================================================================================
// Helper functions for merging OpenAPI specs and serving the combined spec to Scalar UI.
// =======================================================================================

pub async fn serve_merged_auth_spec(auth_openapi_url: String) -> axum::response::Response {
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
        if let Ok(json) = serde_json::to_value(&spec)
            && let Some(obj) = json
                .get("components")
                .and_then(|c| c.get("schemas"))
                .and_then(|s| s.as_object())
        {
            for (k, v) in obj {
                schemas.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }
    serde_json::Value::Object(schemas)
}

pub fn build_docs_html(cfg: &Config) -> String {
    use utoipa::OpenApi;

    let app_spec =
        serde_json::to_string(&AppApiDoc::openapi()).expect("failed to serialize App API spec");

    let system_spec = serde_json::to_string(&SystemApiDoc::openapi())
        .expect("failed to serialize System API spec");

    let auth_base_url = format!(
        "http://localhost:{}{}{}",
        cfg.server.port,
        cfg.server.path_prefix.trim_end_matches('/'),
        cfg.auth.path_prefix,
    );

    let app_base_url = format!(
        "http://localhost:{}{}",
        cfg.server.port,
        cfg.server.path_prefix.trim_end_matches('/'),
    );

    let system_base_url = format!("http://localhost:{}", cfg.server.port,);

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
          {{ 
            title: 'App API', content: {app_spec},
            servers: [{{ url: '{app_base_url}' }}]
          }},
          {{ 
            title: 'System API', content: {system_spec},
            servers: [{{ url: '{system_base_url}' }}]
          }},
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
