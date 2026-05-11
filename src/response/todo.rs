use crate::domain::todo;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct TodoResponse {
    pub id: Uuid,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTodoRequest {
    pub title: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTodoRequest {
    pub title: Option<String>,
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
