use chrono::{DateTime, Utc};

/// Represents the source of a task.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskSource {
    /// Task was created locally within the application.
    Local,
    /// Task was synchronized from an external Jira project.
    Jira,
}

/// The core domain model for a single task or todo item.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    /// Unique internal identifier (UUID).
    pub id: String,
    /// Optional external identifier (e.g., Jira issue key like "PROJ-123").
    pub external_id: Option<String>,
    /// Where this task originated from.
    pub source: TaskSource,
    /// The title of the task.
    pub title: String,
    /// The main description or content of the task.
    pub description: String,
    /// Whether the task is marked as important/prioritized.
    pub important: bool,
    /// Completion status.
    pub completed: bool,
    /// Optional start date for Gantt visualization.
    pub start_date: Option<DateTime<Utc>>,
    /// Optional end/due date for Gantt visualization.
    pub end_date: Option<DateTime<Utc>>,
}

impl Task {
    /// Creates a new local task with default values.
    pub fn new(id: String, title: String, description: String) -> Self {
        Self {
            id,
            external_id: None,
            source: TaskSource::Local,
            title,
            description,
            important: false,
            completed: false,
            start_date: None,
            end_date: None,
        }
    }

    /// Validates that the start date is not after the end date.
    pub fn is_valid_range(&self) -> bool {
        match (self.start_date, self.end_date) {
            (Some(start), Some(end)) => start <= end,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_task_new() {
        let task = Task::new("1".to_string(), "title".to_string(), "test".to_string());
        assert_eq!(task.id, "1");
        assert_eq!(task.title, "title");
        assert_eq!(task.description, "test");
        assert!(!task.completed);
        assert!(!task.important);
        assert_eq!(task.source, TaskSource::Local);
    }

    #[test]
    fn test_is_valid_range() {
        let mut task = Task::new("1".to_string(), "title".to_string(), "test".to_string());
        assert!(task.is_valid_range());

        let now = Utc::now();
        task.start_date = Some(now);
        task.end_date = Some(now + Duration::days(1));
        assert!(task.is_valid_range());

        task.end_date = Some(now - Duration::days(1));
        assert!(!task.is_valid_range());
    }
}
