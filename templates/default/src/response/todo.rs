use crate::domain::todo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// API response for a single todo item.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct TodoResponse {
    /// Unique identifier.
    pub id: Uuid,
    /// Short description of the task.
    pub title: String,
    /// Whether the task has been completed.
    pub done: bool,
    /// When the todo was created (UTC).
    pub created_at: DateTime<Utc>,
}

/// Request body for creating a todo.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateTodoRequest {
    /// Short description of the task (required, non-empty).
    pub title: String,
}

/// Request body for updating a todo.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateTodoRequest {
    /// New title, if changing.
    pub title: Option<String>,
    /// New completion state, if changing.
    pub done: Option<bool>,
}

impl From<todo::Todo> for TodoResponse {
    fn from(t: todo::Todo) -> Self {
        Self {
            id: t.id,
            title: t.title,
            done: t.done,
            created_at: t.created_at,
        }
    }
}

impl From<CreateTodoRequest> for todo::CreateTodo {
    fn from(r: CreateTodoRequest) -> Self {
        Self { title: r.title }
    }
}

impl From<UpdateTodoRequest> for todo::UpdateTodo {
    fn from(r: UpdateTodoRequest) -> Self {
        Self {
            title: r.title,
            done: r.done,
        }
    }
}

impl From<seaorm::store::entities::todos::Model> for todo::Todo {
    fn from(m: seaorm::store::entities::todos::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            done: m.done,
            created_at: m.created_at.into(),
        }
    }
}
