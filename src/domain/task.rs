use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: String,
    pub content: String,
    pub important: bool,
    pub completed: bool,
    pub due_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            content,
            important: false,
            completed: false,
            due_date: None,
        }
    }
}
