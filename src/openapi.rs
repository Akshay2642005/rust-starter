//! OpenAPI 3.1 specification and Scalar UI.
//!
//! Mount with:
//! ```rust,ignore
//! use utoipa_scalar::{Scalar, Servable};
//! let router = router.merge(Scalar::with_url("/docs", ApiDoc::openapi()));
//! ```

use utoipa::{
    Modify, OpenApi,
    openapi::{
        ServerBuilder,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    },
};

use crate::{
    handlers::{
        system::{__path_health, __path_healthz, __path_livez, __path_readyz, __path_status},
        todos::{
            __path_create_todo, __path_delete_todo, __path_get_todo_by_id, __path_get_todos,
            __path_update_todo,
        },
    },
    response::{
        system::{ComponentStatus, ProbeResponse, StatusChecks, StatusResponse},
        todo::{CreateTodoRequest, TodoResponse, UpdateTodoRequest},
    },
};

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        openapi.servers = Some(vec![
            ServerBuilder::new().url("http://localhost:8080/api/v1").build(),
            ServerBuilder::new().url("http://localhost:8080").build(),
        ]);
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Rust Starter API",
        version = "0.1.0",
        description = "Production-ready Rust backend starter — todo service",
        license(name = "MIT"),
    ),
    paths(
        health, healthz, livez, readyz, status,
        create_todo, get_todos, get_todo_by_id, update_todo, delete_todo,
    ),
    components(schemas(
        ProbeResponse, ComponentStatus, StatusChecks, StatusResponse,
        TodoResponse, CreateTodoRequest, UpdateTodoRequest,
    )),
    tags(
        (name = "system", description = "Health and status endpoints"),
        (name = "todos", description = "Todo management endpoints"),
    ),
    modifiers(&BearerAuth),
)]
pub struct ApiDoc;
