use crate::domain::task::Task;
use crate::ports::inbound::TaskServicePort;
use crate::ports::outbound::TaskRepository;
use chrono::{DateTime, Utc};
use std::error::Error;
use std::sync::Arc;

/// Orchestrates domain logic for tasks.
/// 
/// This service acts as the primary entry point for task-related operations,
/// coordinating between the domain model and the persistence/external adapters.
pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
}

impl TaskService {
    /// Creates a new `TaskService` with the given repository adapter.
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }
}

impl TaskServicePort for TaskService {
    fn add_task(
        &self,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<String, Box<dyn Error>> {
        // Domain validation
        let mut temp_task = Task::new("temp".to_string(), title.clone(), description.clone());
        temp_task.start_date = start_date;
        temp_task.end_date = end_date;
        
        if !temp_task.is_valid_range() {
            return Err("Invalid date range: start date must be before end date".into());
        }

        self.repository.add(title, description, start_date, end_date)
    }

    fn get_all_tasks(&self) -> Result<Vec<Task>, Box<dyn Error>> {
        self.repository.get_all()
    }

    fn toggle_completed(&self, id: String) -> Result<(), Box<dyn Error>> {
        self.repository.toggle_completed(id)
    }

    fn toggle_important(&self, id: String) -> Result<(), Box<dyn Error>> {
        self.repository.toggle_important(id)
    }

    fn update_task(
        &self,
        id: String,
        title: String,
        description: String,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut temp_task = Task::new(id.clone(), title.clone(), description.clone());
        temp_task.start_date = start_date;
        temp_task.end_date = end_date;
        
        if !temp_task.is_valid_range() {
            return Err("Invalid date range: start date must be before end date".into());
        }

        self.repository.update_task(id, title, description, start_date, end_date)
    }

    fn remove_task(&self, id: String) -> Result<String, Box<dyn Error>> {
        let success = self.repository.remove(id.clone())?;
        if success {
            Ok(format!("Task {} removed", id))
        } else {
            Err("Task not found".into())
        }
    }

    fn clear_completed_tasks(&self) -> Result<String, Box<dyn Error>> {
        let count = self.repository.clear_completed()?;
        Ok(format!("Cleared {} completed tasks", count))
    }

    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>> {
        self.repository.move_task(id, delta)
    }

    fn sync_jira(&self, config: crate::adapters::tui::config::JiraConfig) -> Result<String, Box<dyn Error>> {
        let jira_adapter = crate::adapters::jira::JiraAdapter::new(config);
        let jira_tasks = jira_adapter.fetch_tasks()?;
        let count = jira_tasks.len();
        
        for task in jira_tasks {
            self.repository.upsert_from_external(task)?;
        }
        
        Ok(format!("Synced {} tasks from Jira", count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::outbound::MockTaskRepository;
    use mockall::predicate::*;

    #[test]
    fn test_add_task_valid() {
        let mut mock_repo = MockTaskRepository::new();
        mock_repo.expect_add()
            .with(eq("title".to_string()), eq("description".to_string()), eq(None), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok("1".to_string()));

        let service = TaskService::new(Arc::new(mock_repo));
        let result = service.add_task("title".to_string(), "description".to_string(), None, None);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "1");
    }

    #[test]
    fn test_add_task_invalid_date_range() {
        let mock_repo = MockTaskRepository::new();
        let service = TaskService::new(Arc::new(mock_repo));
        
        let now = Utc::now();
        let start = Some(now);
        let end = Some(now - chrono::Duration::days(1));
        
        let result = service.add_task("title".to_string(), "description".to_string(), start, end);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date range"));
    }
}
