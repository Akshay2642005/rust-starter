use crate::{domain::todo, state::AppState};
use axum::http::StatusCode;
use seaorm::{
    OrmError,
    sea_orm::{ActiveValue::Set, IntoActiveModel},
    store::entities::todos,
};

#[doc = "Service for creating a new todo item"]
pub async fn create_todo(
    state: AppState,
    input: todo::CreateTodo,
) -> Result<todo::Todo, StatusCode> {
    let active = todos::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set(input.title),
        ..Default::default()
    };

    state
        .db
        .repository::<todos::Entity>()
        .insert(active)
        .await
        .map(todo::Todo::from)
        .map_err(|err| {
            tracing::error!(error = %err, "failed to create todo");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[doc = "Service for retrieving all todo items"]
pub async fn get_todos(state: AppState) -> Result<Vec<todo::Todo>, StatusCode> {
    let todos = state
        .db
        .repository::<todos::Entity>()
        .get_all()
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "failed to retrieve todos");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(todos.into_iter().map(todo::Todo::from).collect())
}

pub async fn get_todo_by_id(state: AppState, id: uuid::Uuid) -> Result<todo::Todo, StatusCode> {
    let model = state
        .db
        .repository::<todos::Entity>()
        .get_by_id(id)
        .await
        .map_err(|err| {
            tracing::error!(
                error = %err,
                "failed to retrieve todo with id {}",
                id
            );

            match err {
                OrmError::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            }
        })?;

    Ok(todo::Todo::from(model))
}

#[doc = "Service for updating an existing todo item by ID"]
pub async fn update_todo(
    state: AppState,
    id: uuid::Uuid,
    input: todo::UpdateTodo,
) -> Result<todo::Todo, StatusCode> {
    let repo = state.db.repository::<todos::Entity>();

    let mut active = repo
        .get_by_id(id)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "failed to retrieve todo with id {}", id);
            StatusCode::NOT_FOUND
        })?
        .into_active_model();

    if let Some(title) = input.title {
        active.title = Set(title);
    }

    if let Some(completed) = input.done {
        active.done = Set(completed);
    }

    repo.update(active)
        .await
        .map(todo::Todo::from)
        .map_err(|err| {
            tracing::error!(error = %err, "failed to update todo with id {}", id);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

#[doc = "Service for deleting a todo item by ID"]
pub async fn delete_todo(state: AppState, id: uuid::Uuid) -> Result<(), StatusCode> {
    let repo = state.db.repository::<todos::Entity>();

    repo.delete_by_id(id).await.map(|_| ()).map_err(|err| {
        tracing::error!(error = %err, "failed to delete todo with id {}", id);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
