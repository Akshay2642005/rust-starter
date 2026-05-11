use crate::auth::AuthenticatedUser;
use crate::{response::todo::*, services::todo as svc, state::AppState};
use axum::{Json, extract::State, http::StatusCode};
use macros::{instrument_handler, route};

#[route(POST, "/todos", protected)]
#[instrument_handler]
pub async fn create_todo(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Json(req): Json<CreateTodoRequest>,
) -> Result<Json<TodoResponse>, StatusCode> {
    let todo = svc::create_todo(state, req.into()).await?;
    Ok(Json(todo.into()))
}
