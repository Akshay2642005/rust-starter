use crate::{domain::todo, response::error::AppError, state::AppState};
use seaorm::{
    OrmError,
    sea_orm::{ActiveValue::Set, IntoActiveModel},
    store::entities::todos,
};

pub async fn create_todo(state: AppState, input: todo::CreateTodo) -> Result<todo::Todo, AppError> {
    if input.title.trim().is_empty() {
        return Err(AppError::bad_request("title must not be empty"));
    }

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
            AppError::internal("failed to create todo")
        })
}

pub async fn get_todos(state: AppState) -> Result<Vec<todo::Todo>, AppError> {
    state
        .db
        .repository::<todos::Entity>()
        .get_all()
        .await
        .map(|rows| rows.into_iter().map(todo::Todo::from).collect())
        .map_err(|err| {
            tracing::error!(error = %err, "failed to retrieve todos");
            AppError::internal("failed to retrieve todos")
        })
}

pub async fn get_todo_by_id(state: AppState, id: uuid::Uuid) -> Result<todo::Todo, AppError> {
    state
        .db
        .repository::<todos::Entity>()
        .get_by_id(id)
        .await
        .map(todo::Todo::from)
        .map_err(|err| match err {
            OrmError::NotFound => AppError::not_found("todo not found"),
            _ => {
                tracing::error!(error = %err, "failed to retrieve todo {}", id);
                AppError::internal("failed to retrieve todo")
            }
        })
}

pub async fn update_todo(
    state: AppState,
    id: uuid::Uuid,
    input: todo::UpdateTodo,
) -> Result<todo::Todo, AppError> {
    let repo = state.db.repository::<todos::Entity>();

    let mut active = repo
        .get_by_id(id)
        .await
        .map_err(|err| match err {
            OrmError::NotFound => AppError::not_found("todo not found"),
            _ => {
                tracing::error!(error = %err, "failed to retrieve todo {}", id);
                AppError::internal("failed to retrieve todo")
            }
        })?
        .into_active_model();

    if let Some(title) = input.title {
        active.title = Set(title);
    }
    if let Some(done) = input.done {
        active.done = Set(done);
    }

    repo.update(active)
        .await
        .map(todo::Todo::from)
        .map_err(|err| {
            tracing::error!(error = %err, "failed to update todo {}", id);
            AppError::internal("failed to update todo")
        })
}

pub async fn delete_todo(state: AppState, id: uuid::Uuid) -> Result<(), AppError> {
    state
        .db
        .repository::<todos::Entity>()
        .delete_by_id(id)
        .await
        .map(|_| ())
        .map_err(|err| match err {
            OrmError::NotFound => AppError::not_found("todo not found"),
            _ => {
                tracing::error!(error = %err, "failed to delete todo {}", id);
                AppError::internal("failed to delete todo")
            }
        })
}
