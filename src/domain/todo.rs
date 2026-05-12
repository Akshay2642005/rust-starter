use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A todo item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Todo {
    /// Unique identifier.
    pub id: uuid::Uuid,
    /// Short description of the task.
    pub title: String,
    /// Whether the task has been completed.
    pub done: bool,
    /// When the todo was created (UTC).
    pub created_at: DateTime<Utc>,
}

/// Payload for creating a new todo.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CreateTodo {
    /// Short description of the task (required, non-empty).
    pub title: String,
}

/// Payload for updating an existing todo.
#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct UpdateTodo {
    /// New title, if changing.
    pub title: Option<String>,
    /// New completion state, if changing.
    pub done: Option<bool>,
}
