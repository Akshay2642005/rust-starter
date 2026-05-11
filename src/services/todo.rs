use crate::{domain::todo, state::AppState};
use axum::http::StatusCode;
use seaorm::{sea_orm::ActiveValue::Set, store::entities::todos};

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
