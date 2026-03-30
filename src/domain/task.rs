use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskSource {
    Local,
    Jira,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: String,
    pub external_id: Option<String>,
    pub source: TaskSource,
    pub content: String,
    pub important: bool,
    pub completed: bool,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(id: String, content: String) -> Self {
        Self {
            id,
            external_id: None,
            source: TaskSource::Local,
            content,
            important: false,
            completed: false,
            start_date: None,
            end_date: None,
        }
    }

    pub fn is_valid_range(&self) -> bool {
        match (self.start_date, self.end_date) {
            (Some(start), Some(end)) => start <= end,
            _ => true,
        }
    }
}
