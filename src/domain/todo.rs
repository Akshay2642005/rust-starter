use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    pub id: uuid::Uuid,
    pub title: String,
    pub done: bool,
    pub created_at: DateTime<Utc>,
}

pub struct CreateTodo {
    pub title: String,
}

#[allow(dead_code)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}
