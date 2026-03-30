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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_task_new() {
        let task = Task::new("1".to_string(), "test".to_string());
        assert_eq!(task.id, "1");
        assert_eq!(task.content, "test");
        assert!(!task.completed);
        assert!(!task.important);
        assert_eq!(task.source, TaskSource::Local);
    }

    #[test]
    fn test_is_valid_range() {
        let mut task = Task::new("1".to_string(), "test".to_string());
        assert!(task.is_valid_range());

        let now = Utc::now();
        task.start_date = Some(now);
        task.end_date = Some(now + Duration::days(1));
        assert!(task.is_valid_range());

        task.end_date = Some(now - Duration::days(1));
        assert!(!task.is_valid_range());
    }
}
