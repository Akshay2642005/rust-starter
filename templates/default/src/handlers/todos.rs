use crate::{response::{error::AppError, todo::*}, services::todo as svc, state::AppState};
use axum::{Json, extract::State};
use macros::{instrument_handler, route};

/// Create a new todo item.
#[utoipa::path(
    post, path = "/todos", tag = "todos",
    request_body = CreateTodoRequest,
    responses(
        (status = 200, description = "Todo created", body = TodoResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_token" = []))
)]
#[instrument_handler]
#[route(POST, "/todos", protected)]
pub async fn create_todo(
    State(state): State<AppState>,
    Json(req): Json<CreateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = svc::create_todo(state, req.into()).await?;
    Ok(Json(todo.into()))
}

/// List all todo items.
#[utoipa::path(
    get, path = "/todos", tag = "todos",
    responses(
        (status = 200, description = "List of todos", body = Vec<TodoResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_token" = []))
)]
#[instrument_handler]
#[route(GET, "/todos", protected)]
pub async fn get_todos(
    State(state): State<AppState>,
) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = svc::get_todos(state).await?;
    Ok(Json(todos.into_iter().map(Into::into).collect()))
}

/// Get a single todo by ID.
#[utoipa::path(
    get, path = "/todos/{id}", tag = "todos",
    params(("id" = uuid::Uuid, Path, description = "Todo ID")),
    responses(
        (status = 200, description = "Todo found", body = TodoResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_token" = []))
)]
#[instrument_handler]
#[route(GET, "/todos/{id}", protected)]
pub async fn get_todo_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = svc::get_todo_by_id(state, id).await?;
    Ok(Json(todo.into()))
}

/// Update an existing todo.
#[utoipa::path(
    put, path = "/todos/{id}", tag = "todos",
    params(("id" = uuid::Uuid, Path, description = "Todo ID")),
    request_body = UpdateTodoRequest,
    responses(
        (status = 200, description = "Todo updated", body = TodoResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_token" = []))
)]
#[instrument_handler]
#[route(PUT, "/todos/{id}", protected)]
pub async fn update_todo(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
    Json(req): Json<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = svc::update_todo(state, id, req.into()).await?;
    Ok(Json(todo.into()))
}

/// Delete a todo by ID.
#[utoipa::path(
    delete, path = "/todos/{id}", tag = "todos",
    params(("id" = uuid::Uuid, Path, description = "Todo ID")),
    responses(
        (status = 204, description = "Todo deleted"),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("bearer_token" = []))
)]
#[instrument_handler]
#[route(DELETE, "/todos/{id}", protected)]
pub async fn delete_todo(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<uuid::Uuid>,
) -> Result<axum::http::StatusCode, AppError> {
    svc::delete_todo(state, id).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
