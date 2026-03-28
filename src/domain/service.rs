use crate::domain::task::Task;
use crate::ports::inbound::TaskServicePort;
use crate::ports::outbound::TaskRepository;
use std::error::Error;
use std::sync::Arc;
use chrono::{DateTime, Utc};

pub struct TaskService {
    repository: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
    }
}

impl TaskServicePort for TaskService {
    fn add_task(&self, content: String, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<String, Box<dyn Error>> {
        let mut task = Task::new("temp".to_string(), content.clone());
        task.start_date = start_date;
        task.end_date = end_date;
        
        if !task.is_valid_range() {
            return Err("Invalid date range: start date must be before or equal to end date".into());
        }

        self.repository.add(content, start_date, end_date)
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

    fn update_task_content(&self, id: String, content: String, start_date: Option<DateTime<Utc>>, end_date: Option<DateTime<Utc>>) -> Result<(), Box<dyn Error>> {
        let mut task = Task::new(id.clone(), content.clone());
        task.start_date = start_date;
        task.end_date = end_date;

        if !task.is_valid_range() {
            return Err("Invalid date range: start date must be before or equal to end date".into());
        }

        self.repository.update_content(id, content, start_date, end_date)
    }

    fn remove_task(&self, id: String) -> Result<String, Box<dyn Error>> {
        let removed = self.repository.remove(id.clone())?;
        if removed {
            Ok(format!("Task {} removed successfully", id))
        } else {
            Ok(format!("Task {} not found", id))
        }
    }

    fn clear_completed_tasks(&self) -> Result<String, Box<dyn Error>> {
        let count = self.repository.clear_completed()?;
        Ok(format!("Cleared {} completed tasks", count))
    }

    fn move_task(&self, id: String, delta: i32) -> Result<(), Box<dyn Error>> {
        self.repository.move_task(id, delta)
    }
}
